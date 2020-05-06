use std::{env, cmp};
use std::time::Instant;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

type Pos = isize;
type Board = Vec<Pos>;
type Nogood = Board;


struct AgentState {
    pos: Board,
    no_goods: Vec<Nogood>,
    oks: Vec<(usize,isize)>,
    pos_seq: Vec<isize>,
    col_i: usize,
}
    
//checks for consistent queen placement
fn consistent(ar: usize, ac: Pos, br: usize, bc: Pos) -> bool {
    // -1 signifies that agent ar is not assigned
    if ac == -1 {
        return true;
    }
    if ac == bc {
        return false;
    }
    if (ar as isize) - (br as isize) == ac - bc {
        return false;
    }
    if (ar as isize) - (br as isize) == bc - ac {
        return false;
    }
    return true;
}


fn eq_part_ass(pa1: &Board, pa2: &Board) -> bool {
    let small_length = cmp::min(pa1.len(), pa2.len());
    for i in 0..small_length {
        
        // -1 means its unassigned
        if pa1[i] == -1 || pa2[i] == -1 {
            continue;
        }
        if pa1[i] != pa2[i] {
            return false;
        }
    }
    return true;
}

fn make_agents(num_agents: usize) -> Vec<AgentState> {
    let mut agents: Vec<AgentState> = vec![];
    let mut pos_vec = vec![];
    let seed = [1; 32];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    for i in 0..(num_agents as isize){pos_vec.push(i);}

    for _ in 0..num_agents {
        pos_vec.shuffle(&mut rng);
        let mut pos_seq = vec![];
        for j in 0..pos_vec.len() {pos_seq.push(pos_vec[j]);}

        let agent = AgentState {
            pos: vec![0; num_agents],
            no_goods: vec![],
            oks: vec![],
            pos_seq: pos_seq,
            col_i: 0,
        };
        agents.push(agent);
    }
    agents
}


fn update_pos(agent: usize, states: &mut Vec<AgentState>,
              num_agents : isize) -> bool {
    let max_pos = num_agents - 1;


    // must check for it being too big here because when we found that a Nogood
    // prevented an otherwise acceptable state, we increment a position,
    // and it could possibly go out of bounds. If we do, we want to send
    // a Nogood to the predecessor. 
    if states[agent].col_i > max_pos as usize{
        states[agent].pos[agent] = states[agent].pos_seq[0];
        states[agent].col_i = 0;
        return false;
    }

    let start = states[agent].col_i;
    let mut found_flag = true;
    for col_i in start..(max_pos as usize + 1) {
        found_flag = true;
        for i in 0..agent {
            if false == consistent(i, states[agent].pos[i], agent,
                                        states[agent].pos_seq[col_i]) {
                found_flag = false;
                break;
            }
        }
        if false == found_flag {continue;}
        states[agent].pos[agent] = states[agent].pos_seq[col_i];
        states[agent].col_i = col_i;
        break;
    }
    if false == found_flag {
        states[agent].pos[agent] = states[agent].pos_seq[0];
        states[agent].col_i = 0;
        return false;
    }

    true
}


// returns true found a consistent assignment
fn run_agent(agent: usize, states: &mut Vec<AgentState>,
             num_agents: isize) -> bool {

    // first update the local view from the ok messages queue.
    // in this sequential version, they're already updated.
    while states[agent].oks.len() > 0 {
        match states[agent].oks.pop() {
            None => break,
            Some(update) => {
                if states[agent].pos[update.0] != update.1 {
                    
                    states[agent].pos[agent] = states[agent].pos_seq[0];
                    states[agent].col_i = 0;
                    states[agent].pos[update.0] = update.1;
                }
            },
        }
    }

    // then look to see if the current agent has a consistent assignment.
    // if not, send a Nogood. If so, check to make sure that it's not ruled
    // out by a Nogood.
    let mut backtrack_depth = 0;
    while false == update_pos(agent, states, num_agents) {
        backtrack_depth = backtrack_depth + 1;
        let pred = ((agent as isize) - backtrack_depth) as usize;

        //send Nogood
        let nogood = states[agent].pos[0..(pred + 1)].to_vec();
        states[pred].no_goods.push(nogood);

        states[agent].pos[agent] = states[agent].pos_seq[0];
        states[agent].col_i = 0;

        // erase agent's belief about its predecessor's position
        states[agent].pos[pred] = -1;

    }
    if backtrack_depth > 0 {return false;}

    // Now that a consistent assignment has been found, check to see if it's
    // ruled out by a Nogood.
    while states[agent].no_goods.len() > 0 {
        match states[agent].no_goods.pop() {
            None => break,
            Some(no_good) =>
                if eq_part_ass(&no_good, &states[agent].pos) {
                    states[agent].col_i += 1;
                    //if it's too big, this will be fixed in update_pos
                    if states[agent].col_i < num_agents as usize {
                        
                        states[agent].pos[agent] =
                            states[agent].pos_seq[states[agent].col_i];
                    }
                    return run_agent(agent, states, num_agents);
                },
        }
    }


    // if the consistent assignment is not ruled out by a Nogood, then you
    // should send ok messages to the other agents
    for succ in (agent + 1)..(num_agents as usize) {
        let new_pos = states[agent].pos[agent];
        states[succ].oks.push((agent, new_pos));
    }
    return true;
}

fn print_board(state: &AgentState, num_agents: isize) {
    let i = num_agents as usize;
    println!("{:?}", state.pos);
    for ii in 0..i {
        for _ in 0..state.pos[ii] {print!("- ");}
        print!("1 ");
        for _ in (state.pos[ii] + 1)..num_agents {
            print!("- ");
        }
        println!();
    }
    println!();

}

fn validate(board: &Board) -> bool {
    for i in 0..(board.len() - 1) {
        for j in (i + 1)..board.len() {
            let diff = (j - i) as isize;
            if board[i] == board[j] {return false;}
            if board[i] - diff == board[j] {return false;}
            if board[i] + diff == board[j] {return false;}
        }
    }
    true
}

fn main() {
    println!("running seq");
    let mut num_agents: isize = 0;
    let mut num_threads: usize;
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if args[i] == "-t" {
            num_threads = args[i + 1].parse::<usize>().unwrap();
        }
        if args[i] == "-a" {
            num_agents = args[i + 1].parse::<isize>().unwrap();
        }
    }

    let now = Instant::now();
    let mut states = make_agents(num_agents as usize);
    let mut found_cons;
    for _ in 0..100000000 {
        found_cons = true;
        for j in 0..(num_agents as usize) {
            found_cons = run_agent(j, &mut states, num_agents)
                            && found_cons;
        }
        if found_cons == true {
            let i = num_agents as usize;
            break;
        }
    }
    println!("{}", now.elapsed().as_micros());
    let i = num_agents as usize;
    if validate(&states[i - 1].pos) {println!("valid");}
    else {println!("not valid")};
}
