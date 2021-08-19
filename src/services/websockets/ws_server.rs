//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Chat server sends this messages to session
#[derive(Message,Serialize,Deserialize)]
#[rtype(result = "()")]
pub struct Message{
    message: String,
    event_type: String,
}

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// Send new to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewSong {
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client id
    pub id: usize,
    /// Room name
    pub name: String,
}

/// leave room.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    /// Client id
    pub id: usize,
    /// Room name
    pub name: String,
}

/// Sync Music, if room does not exists create new one.
#[derive(Message,Debug,Serialize,Deserialize)]
#[rtype(result = "()")]
pub struct SyncMusicPlayPause {
    /// Client id
    pub id: usize,
    /// is Playing
    pub playing: String,
    /// seek 
    pub seek: String,
    /// video ID
    pub video_id: String,
     /// Video index
     pub video_index: String,
    /// Room name
    pub name: String,
}

/// Sync MusicSeek, if room does not exists create new one.
#[derive(Message,Serialize,Deserialize)]
#[rtype(result = "()")]
pub struct SyncMusicSeek {
    /// Client id
    pub id: usize,
    /// Room name
    pub seek: String,
    /// video ID
    pub video_id: String,
    /// Video index
    pub video_index: String,
    /// Room name
    pub name: String,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    host: Option<usize>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("music".to_owned(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            host: None,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
    pub fn send_message(&self, event_type: &str ,room: &str, message: &str, skip_id: usize) {
        println!("Send message, {}",event_type);
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message {
                            event_type:event_type.to_owned(),
                            message:message.to_owned(),
                        });
                    }
                }
            }
        }
    }
    pub fn send_message_to_id(&self,event_type: &str,id:&usize,message: &str) {
        if let Some(addr) = self.sessions.get(id){
            let _ = addr.do_send(Message {
                event_type:event_type.to_owned(),
                message:message.to_owned(),
            }); 
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_message("users","music", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        self.send_message_to_id("welcome",&id, &id.to_string());

        // auto join session to music room
        self.rooms
            .entry("music".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("visitors","music", &format!("Total visitors {}", count), 0);

        // send id back
        id
    }
}

/// Handler for MusicPlayPause message.
impl Handler<SyncMusicPlayPause> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SyncMusicPlayPause, _: &mut Context<Self>) {
        println!("SyncHAndler,{:?}",msg);
        self.send_message("playPause",&msg.name, &serde_json::to_string(&msg).unwrap(), msg.id);
    }
}

/// Handler for MusicPlayPause message.
impl Handler<SyncMusicSeek> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SyncMusicSeek, _: &mut Context<Self>) {
        self.send_message("seek",&msg.name, &serde_json::to_string(&msg).unwrap(), msg.id);
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message("users",&room, "Someone disconnected", 0);

            // make others host
            if room == "sync" && self.host == Some(msg.id) {
                self.host=None;
                if let Some(sessions) = self.rooms.get(&room) {
                    for id in sessions {
                        self.send_message_to_id("host", &id, "true");
                        self.host=Some(*id);
                        break;
                    }
                }
            }
        }
        
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message("message",&msg.room, msg.msg.as_str(), msg.id);
    }
}

/// Handler for Message message.
impl Handler<NewSong> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: NewSong, _: &mut Context<Self>) {
        self.send_message("newSong",&msg.room, msg.msg.as_str(), 0);
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        if name == "sync" && self.host == None {
                self.host=Some(id);
                self.send_message_to_id("host", &id, "true")
        }
        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);
        println!("From inside join: {},{}",id,name);
        self.send_message("users",&name, "Someone Joined", id);
        if name == "sync" {
            self.send_message("syncRoomJoined",&name, "Someone Joined", id);
        }
    }
}

/// Leave room
impl Handler<Leave> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Context<Self>) {
        let Leave { id, name } = msg;

        let mut rooms = Vec::new();

        // TODO: optimize this .. access the room directly without loop
        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if *n == name && sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message("users",&room, "Someone disconnected", 0);
        }

        if name == "sync" && self.host == Some(id) {
            self.host = None;
            if let Some(sessions) = self.rooms.get(&name) {
                for id in sessions {
                    self.send_message_to_id("host", &id, "true");
                    self.host = Some(*id);
                    break;
                }
            }
            self.send_message_to_id("host", &id, "false");
        }

    }
}