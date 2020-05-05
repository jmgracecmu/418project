use std::{env, cmp, thread};
use std::sync::{mpsc, Arc, Barrier};
use std::ops::{Index, IndexMut};
use std::fmt;
use std::time::Instant;
use std::collections::VecDeque;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq)]
enum Position {
    Unass,
    Col(usize),
}

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
enum Message {
    Empty(usize),
    RecvNone,
    Break(usize, usize),
    Ok(usize, Position),
    Nogood(usize, Board),
}

type ID = usize;

struct AgentState {
    id: usize,
    pos: Board,
    cycles: usize,
    end_cycle: usize,
    cycles_with_no_comms: usize,
    no_goods: Vec<Board>,
    txs: Vec<mpsc::Sender<Message>>,
    rx: mpsc::Receiver<Message>,
    mess2send: VecDeque<(usize, Message)>,
    pos_seq: Vec<Position>,
    col_i: usize,
}

impl fmt::Debug for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentState")
        .field("id", &self.id)
        .finish()
    }
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

    let mut pos_vec = vec![];
    let mut rng = rand::thread_rng();
    for i in 0..num_agents {pos_vec.push(i);}

    let (txs, mut rxs) = make_channels(num_agents);
    for i in 0..num_agents {
        pos_vec.shuffle(&mut rng);
        let mut pos_seq = vec![];
        for j in 0..pos_vec.len() {pos_seq.push(Position::Col(pos_vec[j]));}
        if let Some(rx) = rxs.pop() {
            let agent = AgentState {
                id: i,
                pos: Board::Board(vec![Position::Col(0); num_agents]),
                cycles: 0,
                end_cycle: (-1isize) as usize,
                cycles_with_no_comms: 0,
                no_goods: vec![],
                txs: txs.clone(),
                rx: rx,
                mess2send: VecDeque::new(),
                pos_seq: pos_seq,
                col_i: 0,
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
    if state.col_i > max_pos {
        state.pos[state.id] = state.pos_seq[0];
        state.col_i = 0;
        return false;
    }

    let mut start = 0;
    start = state.col_i;
    let mut found_flag = true;
    for col_i in start..(max_pos + 1) {
        found_flag = true;
        // this loop checks to make sure it works with all predecessors
        for i in 0..state.id {
            found_flag = consistent(i, state.pos[i], state.id,
                                                state.pos_seq[col_i]);
            if false == found_flag {break;}
        }
        if false == found_flag {continue;}
        state.pos[state.id] = state.pos_seq[col_i];
        state.col_i = col_i;
        break;
    }
    if false == found_flag {
        state.pos[state.id] = state.pos_seq[0];
        state.col_i = 0;
        return false;
    }

    true
}


fn update_pos(state: &mut AgentState, num_agents:usize) -> bool {
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
        state.mess2send.push_back(
            (pred, Message::Nogood(state.id, Board::Board(nogood)))
        );
        
        /* used to be
        state.txs[pred].send(Message::Nogood(state.id, Board::Board(nogood))).unwrap();
        */
        state.pos[state.id] = state.pos_seq[0];
        state.col_i = 0;

        // erase agent's belief about its predecessor's position
        state.pos[pred] = Position::Unass;

    }
    if backtrack_depth > 0 {
        for i in 0..num_agents {
            if state.id - backtrack_depth <= i && i < state.id {continue;}
            // state.txs[i].send(Message::Empty(state.id)).unwrap();
            state.mess2send.push_back(
                (i, Message::Empty(state.id))
            );
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
    while state.no_goods.len() > 0 {
        match state.no_goods.pop() {
            None => break,
            Some(nogood) => {
                if eq_part_ass(&nogood, &state.pos) {
                    state.col_i += 1;
                    // if it's too big, it will guaranteed to be fixed later
                    // in try_to_inc
                    if state.col_i < num_agents {
                        state.pos[state.id] = state.pos_seq[state.col_i];
                    }
                    return run_agent_rec(state, num_agents);
                }
            },
        }

    }
/*
    for nogood in state.no_goods {
        if eq_part_ass(&nogood, &state.pos) {
            let col: usize;
            if let Position::Col(_col) = state.pos[state.id] {
                col = _col;
            } else {unreachable!();}
            state.pos[state.id] = Position::Col(col + 1);
            return run_agent_rec(state, num_agents);
        }
    }
*/
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
    for succ in (state.id + 1)..num_agents {
        let pos = state.pos[state.id];
        // pos is automatically cloned here. but it's possible I'm trying
        // to move out of a vector. maybe it's cloned above as well
        //state.txs[succ].send(Message::Ok(state.id, pos)).unwrap();
        state.mess2send.push_back((succ, Message::Ok(state.id,pos)));
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
fn recv_messages(num_agents: usize, state: &mut AgentState) -> Message {
    use Message::{Empty, Ok, Nogood, Break};
    
    // every agent needs to receive a Break message to return true
    // except the last agent, which needs to receive no messages to return true
    let mut ret = Message::RecvNone;
    let mut mess_iter = state.rx.try_iter();
    while let Some(mess) = mess_iter.next() {
        match mess {
            Message::RecvNone => {
                unreachable!();
            },
            Message::Empty(sender) => {
                ret = Empty(sender);
            },
            Message::Break(sender, end_cycle) => {
                ret = Break(sender, end_cycle);
                state.end_cycle = end_cycle;
            },
            Message::Ok(sender, pos) => {
                if state.pos[sender] != pos {
                    state.pos[state.id] = state.pos_seq[0];
                    state.col_i = 0;
                    for succ in (state.id + 1)..num_agents {
                        state.mess2send.push_back(
                            (succ, Message::Ok(state.id, Position::Col(0)))
                        );
                    }
                }
                state.pos[sender] = pos;
                ret = Ok(sender, pos);
                ()
            },
            Message::Nogood(sender, nogood) => {
                state.no_goods.push(nogood);
                ret = Nogood(sender, Board::Board(vec![]));
                ()
            },
        };
    }
    ret
}



fn send_messages(state: &mut AgentState) {
    while let Some((dest,mess)) = state.mess2send.pop_front() {
        state.txs[dest].send(mess).unwrap();
    }

}


fn main() {
    println!("running fewer messages");
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
    let now = Instant::now();
    let agents_per_thread = num_agents / num_threads;
    let mut remainder = num_agents % num_threads;
    let mut states = make_agents(num_agents);

    let mut handles = vec![];
    let barrier = Arc::new(Barrier::new(num_threads));
    
    for _ in 0..num_threads {
        let mut local_states = states;
        if remainder > 0 {
            remainder -= 1;
            states = local_states.split_off(agents_per_thread + 1);
        } else {
            states = local_states.split_off(agents_per_thread);
        }

        let c = barrier.clone();
        let handle = thread::spawn(move || {
            let mut recv_ret: Message;
            let mut break_flag = false;
            loop {
                //send messages
                for mut state in &mut local_states {
                    run_agent(&mut state, num_agents);
                    send_messages(&mut state);

                }

                // the barrier must be betweeen sending and receiving
                // to ensure that all messages get
                // sent before we poll for a variable number of messages
                c.wait();
                
                // recv
                for mut state in &mut local_states {
                    recv_ret = recv_messages(num_agents, &mut state);
                    if recv_ret == Message::RecvNone {
                        state.cycles_with_no_comms += 1;
                    } else {
                        state.cycles_with_no_comms = 0;
                    }
                    if state.cycles == state.end_cycle {break_flag = true;}
                    state.cycles += 1;
                }
                if break_flag {break;}
                {
                    let i = local_states.len() - 1;
                    let state = &mut local_states[i];
                    if state.id == num_agents -1
                        && state.cycles_with_no_comms > 10 {
                        for i in 0..num_agents {
                            state.txs[i].send(
                                Message::Break(state.id, state.cycles + 1)).unwrap();
                        }
                    }
                }

                // the only way to receive an idle message is if the last 
                // agent didn't move and has found a solution.
            }
            for state in local_states {
                if state.id == num_agents - 1 {
                    print_board(&state, num_agents)
                }
            }
        });
        handles.push(handle);
    }

    // join
    for handle in handles {
        handle.join().unwrap();
    }
    println!("{:?}", now.elapsed().as_micros());

}
