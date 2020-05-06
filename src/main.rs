use std::{env, cmp, thread};
use std::sync::{mpsc, Arc, Barrier};
use std::ops::{Index, IndexMut};
use std::mem;
use std::time::Instant;

#[derive(Clone, Debug, Copy, PartialEq)]
enum Position {
    Unass,
    Col(usize),
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
enum Message {
    Empty(usize),
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
    mess2send: Vec<Message>,
}
    
//checks for consistent queen placement
fn consistent(ar: ID, ac: Position, br: ID, bc: Position) -> bool {

    match ac {
        Position::Unass => true,
        Position::Col(cola) => match bc {
            Position::Unass => unreachable!(),
            Position::Col(colb) => {
                if cola == colb {return false;}
                if ar + colb == cola + br {return false;}
                if ar + cola == colb + br {return false;}
                true
            }
        }
    }
}


fn eq_part_ass(nogood: &Board, curr_board: &Board) -> bool {
    use Position::{Col, Unass};

    let small_length = cmp::min(nogood.len(), curr_board.len());
    for i in 0..small_length {
        // if either pa1[i] or pa2[i] is Unass, it goes to the next value of i
        // but that's not the behaviour I want. If the predecessor is
        // I suppose that's ok. 
        let _ = match nogood[i] {
            Unass => (),
            Col(col1) => {
                match curr_board[i] {
                    Unass => return false,
                    Col(col2) => if col1 != col2 {return false},
                }
            },
        };
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
                mess2send: vec![Message::Empty(i); num_agents],
            };
            agents.push(agent);
        };
    }
    agents
}


fn try_to_inc_pos(state: &mut AgentState, num_agents : usize) -> bool {
    let max_pos = num_agents - 1;

    // must check for it being too big here because when we found that a Nogood
    // prevented an otherwise acceptable state, we increment a position,
    // and it could possibly go out of bounds. If we do, we want to send
    // a Nogood to the predecessor. 
    if let Position::Col(col) = state.pos[state.id] {
        if col > max_pos {
            state.pos[state.id] = Position::Col(0);
            return false;
        }
    }

    let mut start = 0;
    if let Position::Col(col) = state.pos[state.id] {
        start = col;
    }
    let mut found_flag = true;
    for col in start..(max_pos + 1) {
        found_flag = true;
        // this loop checks to make sure it works with all predecessors
        for i in 0..state.id {
            found_flag = consistent(i, state.pos[i], state.id,
                                                Position::Col(col));
            if false == found_flag {break;}
        }
        if false == found_flag {continue;}
        state.pos[state.id] = Position::Col(col);
        break;
    }
    if false == found_flag {
        state.pos[state.id] = Position::Col(0);
        return false;
    }

    true
}


fn update_pos(state: &mut AgentState, num_agents: usize) -> bool {
    let mut backtrack_depth = 0;
    while false == try_to_inc_pos(state, num_agents) {
        backtrack_depth = backtrack_depth + 1;
        let pred = state.id - backtrack_depth;

        //send Nogood
        let nogood = match &state.pos {
            Board::Board(pos_vec) => pos_vec[0..(pred + 1)].to_vec(),
        };
        // this needs to be a tx
        // used to be states[pred].no_goods.push(nogood);
        state.mess2send[pred] = Message::Nogood(state.id, Board::Board(nogood));
        
        /* used to be
        state.txs[pred].send(Message::Nogood(state.id, Board::Board(nogood))).unwrap();
        */
        state.pos[state.id] = Position::Col(0);

        // erase agent's belief about its predecessor's position
        state.pos[pred] = Position::Unass;

    }
    if backtrack_depth > 0 {
        for i in 0..num_agents {
            if state.id - backtrack_depth <= i && i < state.id {continue;}
            // state.txs[i].send(Message::Empty(state.id)).unwrap();
            state.mess2send[i] = Message::Empty(state.id);
        }
        return false;
    }
    true
}


fn run_agent_rec(state: &mut AgentState, num_agents: usize) -> bool {
    // As noted above, we have received and process the ok messages.
    // the new nogoods are in the vector for later consideration.

    // then look to see if the current agent has a consistent assignment.
    // if not, send a Nogood. 
    if false == update_pos(state, num_agents) {return false;}

    // Now that a consistent assignment has been found, check to see if it's
    // ruled out by a Nogood.
    for nogood in &state.no_goods {
        if eq_part_ass(&nogood, &state.pos) {
            let col: usize;
            if let Position::Col(_col) = state.pos[state.id] {
                col = _col;
            } else {unreachable!();}
            state.pos[state.id] = Position::Col(col + 1);
            return run_agent_rec(state, num_agents);
        }
    }
    true
}


// returns true if the agent did not move or would send idle
// we have the predecessors' new positions from last round and we have the
// successor's nogood from last round, because we have already received
// the messages and updated the preds' positions and the succ's nogood.

fn run_agent(state: &mut AgentState, num_agents: usize) -> bool {
    
    let old_state_col = state.pos[state.id];
    if false == run_agent_rec(state, num_agents) {return false;}

    // if the consistent assignment is not ruled out by a Nogood, then you
    // should send ok messages to the other agents
    if old_state_col != state.pos[state.id] {
        send_oks(state, num_agents);
        return false;
    }

    return true;
}

fn send_oks(state: &mut AgentState, num_agents: usize) {
    use Message::{Empty, Idle, Ok, Nogood};
    for pred in 0..(state.id + 1) {
        //state.txs[pred].send(Message::Empty(state.id)).unwrap();
        state.mess2send[pred] = Message::Empty(state.id);
    }
    for succ in (state.id + 1)..num_agents {
        let pos = state.pos[state.id];
        // pos is automatically cloned here. but it's possible I'm trying
        // to move out of a vector. maybe it's cloned above as well
        //state.txs[succ].send(Message::Ok(state.id, pos)).unwrap();
        state.mess2send[succ] = Message::Ok(state.id, pos);
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
    rxs.reverse();
    (txs, rxs)
}


fn print_board(state : &AgentState, num_agents : usize) {
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
// returns idle iff it receives idle from every other agent
fn receive_messages(num_agents: usize, state: &mut AgentState) -> bool {
    use Message::{Empty, Idle, Ok, Nogood};
    let mut idle = true;
    for i in 0..num_agents {
        let _ = match state.rx.recv().unwrap() {
            Message::Idle(sender) => {
            },
            Message::Empty(sender) => {
                idle = false;
            },
            Message::Ok(sender, pos) => {
                idle = false;
                if state.pos[sender] != pos {
                    state.pos[state.id] = Position::Col(0);
                    for succ in (state.id + 1)..num_agents {
                        state.mess2send[succ] = 
                            Message::Ok(state.id, Position::Col(0));
                    }
                }
                state.pos[sender] = pos;
                ()
            },
            Message::Nogood(sender, nogood) => {
                idle = false;
                state.no_goods.push(nogood);
                ()
            },
        };
    }
    idle
}



fn send_messages(state: &mut AgentState) {
    let mut mess = Message::Idle(state.id);
    for i in 0..state.mess2send.len() {
        mem::swap(&mut state.mess2send[i], &mut mess);
        state.txs[i].send(mess).unwrap();
        mess = Message::Idle(state.id);
    }
}


fn main() {
    println!("running threads v agents");
    let mut num_agents: usize = 0;
    let mut num_threads: usize = 0;
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if args[i] == "-t" {
            num_threads = args[i + 1].parse::<usize>().unwrap();
        }
        if args[i] == "-a" {
            num_agents = args[i + 1].parse::<usize>().unwrap();
        }
    }

    let agents_per_thread = num_agents / num_threads;
    let mut remainder = num_agents % num_threads;
    let mut states = make_agents(num_agents);

    let now = Instant::now();
    let mut handles = vec![];
    let barrier = Arc::new(Barrier::new(num_threads));
    let barrier1 = Arc::new(Barrier::new(num_threads));
    for _ in 0..num_threads {
        let mut local_states = states;
        if remainder > 0 {
            remainder -= 1;
            states = local_states.split_off(agents_per_thread + 1);
        } else {
            states = local_states.split_off(agents_per_thread);
        }
        let c = barrier.clone();
        let c1 = barrier1.clone();
        let handle = thread::spawn(move || {
            let mut idle = false;
            loop {
                // run the agent, including asynchronously
                //sending messages to every other agent
                for mut state in &mut local_states {
                    idle = run_agent(&mut state, num_agents) && idle;
                    send_messages(&mut state);
                }

                idle = true;


                c1.wait();
                // synchronously wait for messages from every 
                //other agent
                for mut state in &mut local_states {
                
                    idle = receive_messages(num_agents, &mut state)
                        && idle;
                }

                if idle {
                    break;
                }


            }
            for state in local_states {
                if state.id == num_agents - 1 {
                    print_board(&state, num_agents)
                }
            }
        });
        handles.push(handle);
    }
    println!("{}", now.elapsed().as_micros());
    // here I think you have to join and determine when to cut the agents off
    for handle in handles {
        handle.join().unwrap();
    }

}
