use std::thread;
use std::sync::{Arc, Mutex, Barrier};
use std::time::Instant;
use std::env;
use std::fs::File;
use std::io::{self,BufReader,BufRead};
use std::path::Path;
use std::str::FromStr;
mod seven_coloring;
pub use crate::seven_coloring::seven_coloring::*;

#[derive(PartialEq)]
enum State{
    NoSol,
    NoChange,
    HasChange,
}

struct Node{
    name: usize,
    color: Option<Color>,
    neighbors: Vec<usize>,
    remaining: Vec<Color>,
    agent_view: Vec<NodeState>,
    no_good: Vec<Color>,
    modified: bool,//keep track of whether the current state has been modified by other nodes
                   //this is equivalent to indicating whether a new msg is received if we are working in message-passing
}

type NodeState = (usize, Color);

impl Node{
    fn new(name:usize) -> Node{
        Node{
            name: name,
            color: None,
            neighbors: Vec::new(),
            remaining: Color::vector_of_colors(),
            agent_view: Vec::new(),
            no_good: Vec::new(),
            modified:false,
        }
    }

    fn add_neighbor(&mut self, neighbor: usize){
        self.neighbors.push(neighbor);
    }

    fn assign_color(&mut self, color: Color){
        self.color = Some(color);
    }
}

//check if the coloring of the input (partial) graph is feasible
//the function makes the assumption that the graph is consists of nodes: 1,2,...,graph.size()
fn all_consistent(graph: &Vec<Node>)->bool{
    for node in graph{
        for neighbor in &node.neighbors{
            if *neighbor < graph.len(){
                if (graph[*neighbor].color).as_ref() == (node.color).as_ref(){
                    return false;
                }
            }
        }
    }
    true
}

//check if cur's assignment is valid
//bound is the number of elements that have been assigned a color in graph
//the fucntion assumes that the original graph is valid
fn new_assign_valid(graph: &Vec<Node>, cur: &Node, bound: usize)->bool{
    for neighbor in &cur.neighbors{
        if *neighbor < bound{
            if (graph[*neighbor].color).as_ref() == (cur.color).as_ref(){
                return false;
            }
        }
    }
    true
}

//single-thread exhaustive search: used as a reference for performance
fn exhaustive_search(graph: &mut Vec<Node>)->bool{
    let mut index = 0;
    while index < graph.len(){
        let mut has_match = false;
        while graph[index].remaining.len() > 0{
            let select = graph[index].remaining.pop().unwrap();
            graph[index].assign_color(select);
            if new_assign_valid(graph, &graph[index], index){
                has_match = true;
                break;
            }
        }
        if has_match == false{
            if index > 0{//backtrack
                //reset color candidates for current vertex before backtracking
                graph[index].remaining = Color::vector_of_colors();
                index = index - 1;
            }else{
                return false;
            }
        }else{//found a match
            index = index + 1;
        }
    }
    true
}

//update color of current node according to agent view
//return the updated color (or None if no color is consistent)
fn update_color(node: &Node) -> Option<Color>{
    let mut valid = true;
    if (node.color != None){//the node already has a color, check whether it is consistent
        for (neighbor_name, neighbor_color) in &node.agent_view{
            if node.color.unwrap() == *neighbor_color{
                valid = false;
            }
        }
        for node_color in &node.no_good{
            if *node_color == node.color.unwrap(){
                valid = false;
            }
        }
    }else{
        valid = false;
    }
    if valid{//no need for a new color
        return node.color;
    }

    //need a new color
    for node_color in &node.remaining{
        //check if color is constrained
        let mut is_constrained = false;
        let mut all_satisfy = true;
        for constrained_color in &node.no_good{
            if *node_color == *constrained_color{
                is_constrained = true;
                break;
            }
        }
        if is_constrained{
            continue;
        }
        for (neighbor_name, neighbor_color) in &node.agent_view{
            if *node_color == *neighbor_color{
                all_satisfy = false;
                break;
            }
        }
        if all_satisfy{
            return Some(*node_color);
        }
    }
    None
}

//single-threaded version of ABT
fn abt_sequential(graph: &mut Vec<Node>) -> bool{
    //let mut iter_index = 0;
    //let mut no_solution = false;
    let mut has_change = true;
    while (has_change){
        has_change = false;
        for node_index in 0..graph.len(){

            //first determine a value

            let (last_color, next_color) = get_last_and_next_color(graph, node_index);
            if next_color != None{
                graph[node_index].assign_color(next_color.unwrap());
            }
            if next_color != None && last_color != None{
                if last_color.unwrap() != next_color.unwrap(){
                    has_change = true;
                }
            }else{
                has_change = true;
            }
            graph[node_index].modified = true;
            //println!("index:{}, color: {:?}", node_index, next_color);
            match next_color{
                //find a value
                Some(color) => {
                    //color is updated, update agent view
                    if last_color == None || last_color.unwrap()==color{
                        for neighbor in graph[node_index].neighbors.clone(){
                            if graph[neighbor].name > node_index{
                                let mut addition = true;
                                for agent_index in 0..graph[neighbor].agent_view.len(){
                                    let (neighbor_name, neighbor_color) = graph[neighbor].agent_view[agent_index];
                                    if neighbor_name == node_index{//agent view of neighbor already contains current node, just need to update value
                                        graph[neighbor].agent_view[agent_index] = (neighbor_name, next_color.unwrap());
                                        addition = false;

                                        break;
                                    }
                                }
                                if addition{//agent view of neighbor does not contain current node, push it
                                    graph[neighbor].agent_view.push((node_index, next_color.unwrap()));
                                }
                                graph[neighbor].modified = true;
                            }

                        }
                    }
                },

                None => {
                    if graph[node_index].agent_view.len() < 1{//no solution and no where to backtrack
                        return false;
                    }else{//backtrack
                        let mut largest_node_index = 0;
                        let mut largest_color:Color = Color::Red;
                        let mut largest_vec_index = 0;
                        for agent_index in 0..graph[node_index].agent_view.len(){
                            let (neighbor_name, neighbor_color) = graph[node_index].agent_view[agent_index];
                            if neighbor_name >= largest_node_index{
                                largest_node_index = neighbor_name;
                                largest_color = neighbor_color;
                                largest_vec_index = agent_index;
                            }
                        }
                        //send nogood
                        graph[largest_node_index].no_good.push(largest_color);
                        graph[largest_node_index].modified = true;
                        graph[node_index].modified = true;
                        graph[node_index].agent_view.remove(largest_vec_index);
                    }
                }
            }

        }

    }
    true
}

fn print_graph(graph: &Vec<Node>){
    for node_index in 0..graph.len(){
        println!("{:?}", graph[node_index].color);
    }
}

fn abt_alg(node: &mut Node){
    if node.name == 0{
        node.assign_color(Color::Red);
    }else if node.name == 1{
        node.assign_color(Color::Blue);
    }else{
        node.assign_color(Color::Green);
    }
}

fn get_last_and_next_color(graph: &mut Vec<Node>, node_index: usize) -> (Option<Color>, Option<Color>){
    let last_color = graph[node_index].color;
            let next_color =
                match last_color{
                    None =>update_color(&graph[node_index]),
                    _ => match graph[node_index].modified{
                        false => graph[node_index].color.clone(),
                        _=> update_color(&graph[node_index]),
                    },
                };
    (last_color, next_color)
}
fn main() {

}

#[cfg(test)]
mod tests{
    use super::*;
    fn gen_larger_graph()->Vec<Node>{
        let v0 = Node::new(0);
        let v1 = Node::new(1);
        let v2 = Node::new(2);
        let v3 = Node::new(3);
        let v4 = Node::new(4);
        let v5 = Node::new(5);
        let v6 = Node::new(6);
        let v7 = Node::new(7);
        let mut graph = vec![v0, v1, v2, v3, v4, v5, v6, v7];
        graph[0].add_neighbor(1);
        graph[0].add_neighbor(4);
        graph[0].add_neighbor(7);
        graph[1].add_neighbor(0);
        graph[1].add_neighbor(2);
        graph[1].add_neighbor(4);
        graph[2].add_neighbor(1);
        graph[2].add_neighbor(3);
        graph[3].add_neighbor(2);
        graph[3].add_neighbor(4);
        graph[3].add_neighbor(5);
        graph[4].add_neighbor(0);
        graph[4].add_neighbor(1);
        graph[4].add_neighbor(3);
        graph[4].add_neighbor(5);
        graph[4].add_neighbor(7);
        graph[5].add_neighbor(3);
        graph[5].add_neighbor(4);
        graph[5].add_neighbor(6);
        graph[6].add_neighbor(5);
        graph[6].add_neighbor(7);
        graph[7].add_neighbor(0);
        graph[7].add_neighbor(4);
        graph[7].add_neighbor(6);
        graph
    }

    fn gen_nosol_graph()->Vec<Node>{
        let v0 = Node::new(0);
        let v1 = Node::new(1);
        let v2 = Node::new(2);
        let v3 = Node::new(3);
        let v4 = Node::new(4);
        let v5 = Node::new(5);
        let v6 = Node::new(6);
        let v7 = Node::new(7);
        let mut graph = vec![v0, v1, v2, v3, v4, v5, v6, v7];
        graph[0].add_neighbor(1);
        graph[0].add_neighbor(4);
        graph[0].add_neighbor(7);
        graph[1].add_neighbor(0);
        graph[1].add_neighbor(2);
        graph[1].add_neighbor(4);
        graph[2].add_neighbor(1);
        graph[2].add_neighbor(3);
        graph[3].add_neighbor(2);
        graph[3].add_neighbor(4);
        graph[3].add_neighbor(5);
        graph[4].add_neighbor(0);
        graph[4].add_neighbor(1);
        graph[4].add_neighbor(3);
        graph[4].add_neighbor(5);
        graph[4].add_neighbor(6);
        graph[4].add_neighbor(7);
        graph[5].add_neighbor(3);
        graph[5].add_neighbor(4);
        graph[5].add_neighbor(6);
        graph[6].add_neighbor(5);
        graph[6].add_neighbor(7);
        graph[7].add_neighbor(0);
        graph[7].add_neighbor(4);
        graph[7].add_neighbor(6);
        graph
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn read_graph(filename: &str) -> Vec<Node>{
        //let file = File::open(filename).unwrap();
        //let reader = BufReader::new(file);
        let mut graph = Vec::new();
        if let Ok(lines) = read_lines(filename) {
                // Consumes the iterator, returns an (Optional) String
                for line in lines {
                    if let Ok(ip) = line {
                        let slices = ip.split(" ").collect::<Vec<&str>>();
                        if slices[0] == "p"{//the line contains information about number of nodes/edges
                            let num_nodes: usize = slices[2].parse().unwrap();
                            for i in 0..num_nodes{
                                graph.push(Node::new(i));
                            }
                        }else if slices[0] == "e"{
                            let source: usize = slices[1].parse().unwrap();
                            let sink: usize = slices[2].parse().unwrap();
                            graph[source-1].add_neighbor(sink-1);
                            graph[sink-1].add_neighbor(source-1);
                            //graph[sink].add_neighbor(source);
                        }

                    }
                }
            }
        //let f2 = reader.get_ref();
        graph
    }

    #[test]
    fn file_test(){
        read_graph("src/myciel3.sol");
    }
    #[test]
    fn simple_c3(){
        let v1 = Node::new(0);
        let v2 = Node::new(1);
        let v3 = Node::new(2);
        let mut graph = vec![v1, v2, v3];
        graph[0].add_neighbor(1);
        graph[0].add_neighbor(2);
        graph[1].add_neighbor(2);
        graph[1].add_neighbor(0);
        graph[2].add_neighbor(0);
        graph[2].add_neighbor(1);
        assert_eq!(exhaustive_search(&mut graph),true);
        assert_eq!(all_consistent(&graph), true);
    }

    #[test]
    fn simple_invalid(){
        let mut graph = Vec::new();
        let num_nodes = Color::num_colors()+1;
        for i in 0..num_nodes{
            graph.push(Node::new(i));
        }

        for node_index in 0..num_nodes{
            for neighbor_index in 0..num_nodes{
                if node_index != neighbor_index{
                    graph[node_index].add_neighbor(neighbor_index);
                }
            }
        }

    }

    #[test]
    fn larger_graph_abt_seq(){
        let mut graph = gen_larger_graph();
        let now = Instant::now();
        assert_eq!(abt_sequential(&mut graph), true);
        let new_now = Instant::now();
        println!("abt seq duration: {:?}", new_now.duration_since(now));
    }

    #[test]
    fn larger_graph_exhaustive(){
        let mut graph = gen_larger_graph();
        let now = Instant::now();
        assert_eq!(exhaustive_search(&mut graph), true);
        let new_now = Instant::now();
        println!("abt exhaustive duration: {:?}", new_now.duration_since(now));
    }



    #[test]
    fn thread_test(){
        //a toy example with thread
        let v0 = Node::new(0);
        let v1 = Node::new(1);
        let v2 = Node::new(2);
        let graph = Arc::new(Mutex::new(vec![v0, v1, v2]));

        let mut handles = Vec::new();
        for i in 0..3{
            //graph_copy and graph will be pointing to the same memory location
            let graph_copy = Arc::clone(&graph);
            let handle = thread::spawn(move||{
                //pass in the i-th node
                //abt_alg will modify the color of the node that gets passed in
                abt_alg(&mut graph_copy.lock().unwrap()[i]);
            });
            handles.push(handle);
        }
        //wait for all threads
        for handle in handles{
            handle.join();
        }
        //check if the colors are assigned correctly
        //for item in graph.lock().unwrap().iter(){
        //    println!("current color is {:?}", item.color);
        //}
    }

    #[test]
    fn ciel_exhaustive(){
        let mut graph = read_graph("src/myciel5.sol");
        let now = Instant::now();
        exhaustive_search(&mut graph);
        let new_now = Instant::now();

        println!("ciel exhaustive: duration: {:?}", new_now.duration_since(now));

    }

    #[test]
    fn ciel_sequential(){
        let mut graph = read_graph("src/myciel7.sol");
        let now = Instant::now();
        abt_sequential(&mut graph);
        let new_now = Instant::now();

        println!("ciel sequential: duration: {:?}", new_now.duration_since(now));
    }

    #[test]
    fn ciel_parallel(){
        let num_agents = 8;
        let mut graph = read_graph("src/myciel7.sol");
        //first, partition the graph
        let num_nodes = graph.len();
        let agent_per_thread_upper: usize = (num_nodes + num_agents - 1) / num_agents;
        let agent_per_thread_lower: usize = num_nodes / num_agents;
        let lower_start: usize =
            match agent_per_thread_upper == agent_per_thread_lower{
                true => num_agents,
                false => num_nodes % num_agents,
            };


        let graph = Arc::new(Mutex::new(graph));
        let barrier = Arc::new(Barrier::new(num_agents));
        let mut handles = Vec::new();
        let stop = Arc::new(Mutex::new(false));
        let idle_threads = Arc::new(Mutex::new(0));
        let now = Instant::now();

        for i in 0..num_agents{
            let graph_copy = Arc::clone(&graph);
            let barrier_copy = barrier.clone();
            let stop_copy = stop.clone();
            let idle_threads_copy = idle_threads.clone();
            let handle = thread::spawn(move||{
                let start_index =
                    match i < lower_start{
                        true => i * agent_per_thread_upper,
                        false => lower_start * agent_per_thread_upper + (i-lower_start)*agent_per_thread_lower,
                };
                let end_index = {
                    match i < lower_start{
                        true => start_index + agent_per_thread_upper,
                        false => start_index + agent_per_thread_lower,
                    }
                };

                loop{
                    let mut has_change = false;
                    for node_index in start_index..end_index{
                        //println!("{}", node_index);
                        let (last_color, next_color) = get_last_and_next_color(&mut graph_copy.lock().unwrap(), node_index);
                        if next_color != None{
                            graph_copy.lock().unwrap()[node_index].assign_color(next_color.unwrap());
                        }
                        if next_color != None && last_color != None{
                            if last_color.unwrap() != next_color.unwrap(){
                                has_change = true;
                            }
                        }else{
                            has_change = true;
                        }
                        graph_copy.lock().unwrap()[node_index].modified = true;
                        match next_color{
                            //find a value
                            Some(color) => {
                                //color is updated, update agent view
                                if last_color == None || last_color.unwrap()==color{
                                    let mut graph = graph_copy.lock().unwrap();
                                    for neighbor in graph[node_index].neighbors.clone(){
                                        if graph[neighbor].name > node_index{
                                            let mut addition = true;
                                            for agent_index in 0..graph[neighbor].agent_view.len(){
                                                let (neighbor_name, neighbor_color) = graph[neighbor].agent_view[agent_index];
                                                if neighbor_name == node_index{//agent view of neighbor already contains current node, just need to update value
                                                    graph[neighbor].agent_view[agent_index] = (neighbor_name, next_color.unwrap());
                                                    addition = false;

                                                    break;
                                                }
                                            }
                                            if addition{//agent view of neighbor does not contain current node, push it
                                                graph[neighbor].agent_view.push((node_index, next_color.unwrap()));
                                            }
                                            graph[neighbor].modified = true;
                                        }

                                    }
                                }
                            },

                            None => {
                                let mut graph = graph_copy.lock().unwrap();
                                if graph[node_index].agent_view.len() < 1{//no solution and no where to backtrack
                                    let mut stop = stop_copy.lock().unwrap();
                                    *stop = true;
                                    break;
                                }else{//backtrack
                                    let mut largest_node_index = 0;
                                    let mut largest_color:Color = Color::Red;
                                    let mut largest_vec_index = 0;
                                    for agent_index in 0..graph[node_index].agent_view.len(){
                                        let (neighbor_name, neighbor_color) = graph[node_index].agent_view[agent_index];
                                        if neighbor_name >= largest_node_index{
                                            largest_node_index = neighbor_name;
                                            largest_color = neighbor_color;
                                            largest_vec_index = agent_index;
                                        }
                                    }
                                    //send nogood
                                    graph[largest_node_index].no_good.push(largest_color);
                                    graph[largest_node_index].modified = true;
                                    graph[node_index].modified = true;
                                    graph[node_index].agent_view.remove(largest_vec_index);
                                }
                            }
                        }

                    }
                    if !has_change{
                        let mut idle_threads = idle_threads_copy.lock().unwrap();
                        *idle_threads = *idle_threads + 1;
                    }
                    barrier_copy.wait();
                    if *(idle_threads_copy.lock().unwrap()) >= 8{
                        break;
                    }else{
                        let mut idle_threads = idle_threads_copy.lock().unwrap();
                        *idle_threads = 0;
                    }
                    if *(stop_copy.lock().unwrap()){
                        break;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles{
            handle.join();
        }
        let new_now = Instant::now();
        println!("ciel par: {:?}", new_now.duration_since(now));

    }
}
