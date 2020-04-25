type Nogood = Vec<u8>;

struct AgentState {
    id: u8,
    pos: Vec<u8>,
    no_goods: Vec<Nogood>,
    new_no_goods: Vec<Nogood>,
    oks: Vec<(u8,u8)>,
}
    
//checks for consistent queen placement
fn consistent(ar: u8, ac: u8, br: u8, bc: u8) {
    // -1 signifies that agent ar is not assigned
    if ac == -1 {
        return true;
    }
    if ac == bc {
        return false;
    }
    if ar - ac == br - bc {
        return false;
    }
    if ar - ac == bc - br {
        return false;
    }
    return true;
}


fn eq_part_ass(pa1: Vec<u8>, pa2: Vec<u8>) -> bool {
    let small_length = min(pa1.len(), pa2.len());
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


fn update_pos(agent: u8, states: Vec<AgentState>) -> bool {
    let max_pos = 3;

    // must check for it being too big here because when we found that a Nogood
    // prevented an otherwise acceptable state, we increment a position,
    // and it could possibly go out of bounds. If we do, we want to send
    // a Nogood to the predecessor. 
    if states[agent].pos[agent] < max_pos {
        states[agent].pos[agent] = 0;
        return false;
    }
 
    // select position that is consistent with predecessors
    for i in 0..agent {
        while false == consistent(i, states[agent].pos[i],
                                agent, states[agent].pos[agent]) {
            states[agent].pos[agent] = states[agent].pos[agent] + 1;
            if states[agent].pos[agent] > max_pos {
                states[agent].pos[agent] = 0;
                return false;
            }
        }

    }
    true
}


fn run_agent(agent: u8, states: Vec<AgentState>) {
    // first update the local view from the ok messages queue.
    // in this sequential version, they're already updated.
    // then look to see if the current agent has a consistent assignment.
    // if not, send a Nogood. If so, check to make sure that it's not ruled
    // out by a Nogood.
    if false == update_pos(agent, states) {
        // must send no good to lowest affected member
        // I think that in the queens problem it will always be the immediate
        // predecessor
        // also assume that the lowest affected member changes its position
        // we don't know what it gets changed to, but remove it as a conflict
        let imm_pred = agent - 1;

        //send Nogood
        let nogood = states[agent].pos[0..agent].to_vec();
        states[imm_pred].no_goods.push(nogood);

        // erase agent's belief about its predecessor's position
        states[agent].pos[imm_pred] = -1;
        if false == update_pos(agent, states) {
            panic!("failed to get working position");
        }
        // if you send a Nogood, you don't send an ok message
        return;
    }
    // Now that a consistent assignment has been found, check to see if it's
    // ruled out by a Nogood.

    while(states[agent].no_goods.len() > 0) {
        let no_good = states[agent].no_goods.pop();
        if eq_part_ass(no_good, states[agent].pos) {
            states[agent].pos[agent] = states[agent].pos[agent] + 1;
            run_agent(agent, states);
            return;
        }
        

    }

}


fn make_agents(num_agents: u8) -> Vec<AgentState> {
    let mut agents: Vec<AgentState> = vec![];
    for i in 0..num_agents {
        let agent = AgentState {
            id: i,
            pos: vec![0u8; num_agents],
            no_goods: vec![],
            new_no_goods: vec![],
            oks: vec![],
        }
        agents.push(agent);
    }
    agents
}

fn main() {
    let num_agents = 4;
    let agents = make_agents(num_agents);
}
