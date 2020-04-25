use std::cmp;


type Pos = isize;
type Board = Vec<Pos>;
type Nogood = Board;


struct AgentState {
    id: usize,
    pos: Board,
    no_goods: Vec<Nogood>,
    new_no_goods: Vec<Nogood>,
    oks: Vec<(usize,isize)>,
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
    for i in 0..num_agents {
        let agent = AgentState {
            id: i,
            pos: vec![0; num_agents],
            no_goods: vec![],
            new_no_goods: vec![],
            oks: vec![],
        };
        agents.push(agent);
    }
    agents
}


fn update_pos(agent: usize, states: &mut Vec<AgentState>,
              num_agents : isize) -> bool {
    let max_pos = num_agents - 1;

    println!("update pos agent {}", agent);

    // must check for it being too big here because when we found that a Nogood
    // prevented an otherwise acceptable state, we increment a position,
    // and it could possibly go out of bounds. If we do, we want to send
    // a Nogood to the predecessor. 
    if states[agent].pos[agent] > max_pos {
        states[agent].pos[agent] = 0;
        println!("too big from finding Nogood earlier");
        println!("return agent {}, pos {}", agent, states[agent].pos[agent]);
        return false;
    }

    let start = states[agent].pos[agent];
    let mut found_flag = true;
    for pos in start..(max_pos + 1) {
        found_flag = true;
        for i in 0..agent {
            if false == consistent(i, states[agent].pos[i], agent, pos) {
                found_flag = false;
                break;
            }
        }
        if false == found_flag {continue;}
        states[agent].pos[agent] = pos;
        break;
    }
    if false == found_flag {
        states[agent].pos[agent] = 0;
        println!("return agent {}, pos {}", agent, states[agent].pos[agent]);
        return false;
    }

    println!("return agent {}, pos {}", agent, states[agent].pos[agent]);
    true
}


// returns true found a consistent assignment
fn run_agent(agent: usize, states: &mut Vec<AgentState>,
             num_agents: isize) -> bool {
    println!("run_agent agent {}", agent);

    // first update the local view from the ok messages queue.
    // in this sequential version, they're already updated.
    while states[agent].oks.len() > 0 {
        match states[agent].oks.pop() {
            None => break,
            Some(update) => {
                if states[agent].pos[update.0] != update.1 {
                    
                    states[agent].pos[agent] = 0;
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
        println!("nogood");

        states[agent].pos[agent] = 0;

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
                    println!("found outruling nogood");
                    states[agent].pos[agent] = states[agent].pos[agent] + 1;
                    return run_agent(agent, states, num_agents);
                },
        }
    }


    // if the consistent assignment is not ruled out by a Nogood, then you
    // should send ok messages to the other agents
    println!("bdcsting updated state");
    for succ in (agent + 1)..(num_agents as usize) {
        let new_pos = states[agent].pos[agent];
        states[succ].oks.push((agent, new_pos));
    }
    return true;
}



fn main() {
    let num_agents : isize = 8;
    let mut states = make_agents(num_agents as usize);
    let mut found_cons;
    for _ in 0..100 {
        found_cons = true;
        println!();
        println!();
        for j in 0..(num_agents as usize) {
            println!();
            found_cons = run_agent(j, &mut states, num_agents) & found_cons;
        }
        if found_cons == true {break;}
    }
}
