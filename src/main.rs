use std::{cmp, thread};
use std::sync::{mpsc, Arc, Barrier};
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, Copy)]
enum Position {
    Unass,
    Col(usize),
}

#[derive(Debug)]
enum Board {
    Board(Vec<Position>),
}

impl Board {
    fn len(&self) -> usize {
        match self {
            Board::Board(pos_vec) => pos_vec.len(),
        }
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, i: usize) -> &mut Position {
        match self {
            Board::Board(pos_vec) => &mut pos_vec[i],
        }
    }
}



impl Index<usize> for Board {
    type Output = Position;

    fn index(& self, i: usize) -> &Self::Output {
        match self {
            Board::Board(pos_vec) => &pos_vec[i],
        }
    }
}



// a message can hold either an update position or a Nogood
enum Message {
    Idle(usize),
    Ok(usize, Position),
    Nogood(usize, Board),
}

type ID = usize;

struct AgentState {
    id: usize,
    pos: Board,
    no_goods: Vec<Board>,
    txs: Vec<mpsc::Sender<Message>>,
    rx: mpsc::Receiver<Message>,
}
    
//checks for consistent queen placement
fn consistent(ar: ID, ac: Position, br: ID, bc: Position) -> bool {

    match ac {
        Position::Unass => true,
        Position::Col(cola) => match bc {
            Position::Unass => true,
            Position::Col(colb) => {
                if cola == colb {return false;}
                if ar + colb == cola + br {return false;}
                if ar + cola == colb + br {return false;}
                true
            }
        }
    }
}


fn eq_part_ass(pa1: &Board, pa2: &Board) -> bool {
    use Position::Col;

    let small_length = cmp::min(pa1.len(), pa2.len());
    for i in 0..small_length {
        // if either pa1[i] or pa2[i] is Unass, it goes to the next value of i
        if let Col(col1) = pa1[i] {
            if let Col(col2) = pa2[i] {
                if col1 != col2 {return false;}
            }
        }
    }
    return true;
}

fn make_agents(num_agents: usize) -> Vec<AgentState> {
    let mut agents: Vec<AgentState> = vec![];
    let (mut txs, mut rxs) = make_channels(num_agents);
    for i in 0..num_agents {
        if let Some(rx) = rxs.pop() {
            let agent = AgentState {
                id: i,
                pos: Board::Board(vec![Position::Col(0); num_agents]),
                no_goods: vec![],
                txs: txs.clone(),
                rx: rx,
            };
            agents.push(agent);
        };
    }
    agents
}


fn update_pos(agent: ID, state: &mut AgentState, num_agents : usize) -> bool {
    let max_pos = num_agents - 1;


    // must check for it being too big here because when we found that a Nogood
    // prevented an otherwise acceptable state, we increment a position,
    // and it could possibly go out of bounds. If we do, we want to send
    // a Nogood to the predecessor. 
    if let Position::Col(col) = state.pos[agent] {
        if col > max_pos {
            state.pos[agent] = Position::Col(0);
            return false;
        }
    }

    let mut start = 0;
    if let Position::Col(col) = state.pos[agent] {
        start = col;
    }
    let mut found_flag = true;
    for col in start..num_agents {
        found_flag = true;
        for i in 0..agent {
            if false == consistent(i, state.pos[i], agent, Position::Col(col)) {
                found_flag = false;
                break;
            }
        }
        if false == found_flag {continue;}
        state.pos[agent] = Position::Col(col);
        break;
    }
    if false == found_flag {
        state.pos[agent] = Position::Col(0);
        return false;
    }

    true
}


// returns true if the agent did not move or would send idle
// we have the predecessors' new positions from last round and we have the
// successor's nogood from last round, because we have already received
// the messages and updated the preds' positions and the succ's nogood.

fn run_agent(agent: usize, state: &mut AgentState, num_agents: usize) -> bool {

    // As noted above, we have received and process the ok messages.
    // the new nogoods are in the vector for later consideration.

    // then look to see if the current agent has a consistent assignment.
    // if not, send a Nogood. 
    let mut backtrack_depth = 0;
    while false == update_pos(agent, state, num_agents) {
        backtrack_depth = backtrack_depth + 1;
        let pred = agent - backtrack_depth;

        //send Nogood
        let nogood = match &state.pos {
            Board::Board(pos_vec) => pos_vec[0..(pred + 1)].to_vec(),
        };
        // this needs to be a tx
        // used to be states[pred].no_goods.push(nogood);
        state.txs[pred].send(Message::Nogood(agent, Board::Board(nogood)));
        println!("{:?} send to {:?}", agent, pred);

        state.pos[agent] = Position::Col(0);

        // erase agent's belief about its predecessor's position
        state.pos[pred] = Position::Unass;

    }
    if backtrack_depth > 0 {
        for i in 0..num_agents {
            if agent - backtrack_depth <= i && i < agent {continue;}
            println!("{:?} send to {:?}", agent, i);
            state.txs[i].send(Message::Idle(agent));
        }
        return false;
    }

    // Now that a consistent assignment has been found, check to see if it's
    // ruled out by a Nogood.
    for nogood in &state.no_goods {
        if eq_part_ass(&nogood, &state.pos) {
            let col: usize;
            if let Position::Col(_col) = state.pos[agent] {
                col = _col;
            } else {unreachable!();}
            state.pos[agent] = Position::Col(col + 1);
            return run_agent(agent, state, num_agents);
        }
    }


    // if the consistent assignment is not ruled out by a Nogood, then you
    // should send ok messages to the other agents
    send_oks(agent, state, num_agents);

    return true;
}

fn send_oks(agent: usize, state: &AgentState, num_agents: usize) {
    for pred in 0..(agent + 1) {
        println!("{:?} send to {:?}", agent, pred);
        state.txs[pred].send(Message::Idle(agent));
    }
    for succ in (agent + 1)..num_agents {
        println!("{:?} send to {:?}", agent, succ);
        let pos = state.pos[agent];
        // pos is automatically cloned here. but it's possible I'm trying
        // to move out of a vector. maybe it's cloned above as well
        state.txs[succ].send(Message::Ok(agent, pos));
    }

}



fn make_channels(num_agents : usize)
        -> (Vec::<mpsc::Sender<Message>>, Vec::<mpsc::Receiver<Message>>) {
    let mut txs = Vec::< mpsc::Sender<Message> >::new();
    let mut rxs = Vec::< mpsc::Receiver<Message> >::new();
    for _ in 0..num_agents {
        let (tx, rx) = mpsc::channel();
        txs.push(tx);
        rxs.push(rx);
    }
    (txs, rxs)
}


fn print_board(state : AgentState, num_agents : usize) {
    let i = num_agents;
    println!("{:?}", state.pos);
    for ii in 0..i {
        if let Position::Col(col) = state.pos[ii] {
            for _ in 0..col {print!("-");}
            print!("1");
            for _ in (col + 1)..num_agents {print!("-");}
        }
        println!();
    }
    println!();
} 

// receive messages. Updates local view and puts nogoods in the vector
fn receive_messages(i: usize, num_agents: usize, state: &mut AgentState)
                            -> bool {
    let mut idle = true;
    for agentid in 0..num_agents {
        println!("agent {:?} waiting on message {:?}", i, agentid);
        let _ = match state.rx.recv().unwrap() {
            Message::Idle(sender) => println!("{:?} recv from {:?}", i, sender),
            Message::Ok(sender, pos) => {
                idle = false;
                state.pos[sender] = pos;
                println!("{:?} recv from {:?}", i, sender);
                ()
            },
            Message::Nogood(sender, nogood) => {
                idle = false;
                state.no_goods.push(nogood);
                println!("{:?} recv from {:?}", i, sender);
                ()
            },
        };
    }
    idle
}



fn main() {
    let num_agents = 4 as usize;
    let mut states = make_agents(num_agents);

    let mut handles = vec![];
    let barrier = Arc::new(Barrier::new(num_agents));
    let barrier1 = Arc::new(Barrier::new(num_agents));
    for _ in 0..num_agents {
        let c = barrier.clone();
        let c1 = barrier1.clone();
        let _ = match states.pop() {
            None => (),
            Some(mut state) => {
                let handle = thread::spawn(move || {
                    let mut idle = true;
                    let i = state.id;
                    loop {
                        c.wait();
                        println!("agent {:?}", state.id); 
                        // run the agent, including asynchronously
                        //sending messages to every other agent
                        idle = idle && run_agent(i, &mut state, num_agents);

                        c1.wait();
                        // synchronously wait for messages from every 
                        //other agent
                        idle = receive_messages(i, num_agents, &mut state);
                        if idle {
                            println!("agent {:?} idle", i);
                        }


                    }
                });
                handles.push(handle);
                ()
            },
        };
    }

    // here I think you have to join and determine when to cut the agents off
    for handle in handles {
        handle.join().unwrap();
    }

}
