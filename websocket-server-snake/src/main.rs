use actix::Arbiter;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::sync::{RwLock, Arc};
use actix::clock::Duration;
use std::time::SystemTime;
use crate::websocket::{Player, SessionContainer, PlayersLobby, snake_game};

mod websocket {
    use std::collections::{HashMap, HashSet};
    use actix::{Addr, Actor, Running, Message, Handler, StreamHandler, ActorContext, AsyncContext};
    use actix_web::web;
    use std::sync::{RwLock, Arc};
    use actix_web_actors::ws;
    use std::time::SystemTime;
    use actix::clock::Duration;
    use snake::Direction;
    use snake::world::{WorldView, SnakeInfo, CreateError, ObjectType};
    use std::thread;
    use std::rc::Rc;
    use std::cell::RefCell;
    use bytes::{Bytes, BytesMut, BufMut};
    use std::task::Context;
    use actix_web_actors::ws::ProtocolError;

    /// - Packet

    struct Packet {
        id: u8,
        data: Vec<u8>
    }

    impl Packet {
        fn from_bytes(bytes: Bytes) -> Option<Self> {
            if bytes.len() < 1 {
                return None
            }
            let (id_part, data_part) = bytes.split_at(1);
            let id = id_part.first().unwrap().clone();
            let data = data_part.to_vec();
            Some(Packet {
                id,
                data
            })
        }
    }

    /// - ClientPacket

    enum ClientPacket {
        Direction(Option<Direction>)
    }

    impl ClientPacket {
        fn from_bytes(bytes: Bytes) -> Option<Self> {
            let packet = Packet::from_bytes(bytes)?;
            match packet.id  {
                199 => Some(ClientPacket::Direction(match packet.data.first() {
                    Some(1) => Some(Direction::Right),
                    Some(2) => Some(Direction::Left),
                    Some(3) => Some(Direction::Down),
                    Some(4) => Some(Direction::Up),
                    _ => None,
                })),
                _ => None
            }
        }
    }

    /// - PlayersLobby

    pub struct PlayersLobby {
        pub players: Vec<Arc<RwLock<Player>>>
    }

    impl Default for PlayersLobby {
        fn default() -> Self {
            PlayersLobby {
                players: Vec::new()
            }
        }
    }

    /// - Player

    pub struct Player {
        address: Option<Addr<SessionContainer<Self>>>,
        direction: Option<Direction>
    }

    impl Default for Player {
        fn default() -> Self {
            Player {
                address: None,
                direction: None
            }
        }
    }

    impl Player {
        fn is_active(&self) -> bool {
            self.address.is_some()
        }
    }

    impl SessionObject for Player {
        fn session_started(
            &mut self,
            ctx: &mut ws::WebsocketContext<SessionContainer<Self>>
        ) {
            self.address = Some(ctx.address());
        }

        fn session_message(
            &mut self,
            msg: Result<ws::Message, ws::ProtocolError>,
            ctx: &mut ws::WebsocketContext<SessionContainer<Self>>
        ) {
            match msg {
                Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
                // Ok(ws::Message::Text(text)) => ctx.text(text),
                Ok(ws::Message::Binary(bytes)) => {
                    let client_packet = ClientPacket::from_bytes(bytes);
                    match client_packet {
                        Some(ClientPacket::Direction(direction)) => {
                            self.direction = direction;
                        }
                        None => {}
                    }
                },
                Ok(ws::Message::Close(_)) => ctx.stop(),
                _ => (),
            }
        }

        fn session_stopped(
            &mut self,
            _ctx: &mut ws::WebsocketContext<SessionContainer<Self>>
        ) {
            self.address = None;
        }
    }

    /// - SessionContainer

    pub trait SessionObject: Sized {
        fn session_started(
            &mut self,
            ctx: &mut ws::WebsocketContext<SessionContainer<Self>>
        );
        fn session_message(
            &mut self,
            msg: Result<ws::Message, ws::ProtocolError>,
            ctx: &mut ws::WebsocketContext<SessionContainer<Self>>
        );
        fn session_stopped(
            &mut self,
            ctx: &mut ws::WebsocketContext<SessionContainer<Self>>
        );
    }

    pub struct SessionContainer<T> where
        T: SessionObject,
        T: 'static {
        object: Arc<RwLock<T>>
    }

    impl<T> SessionContainer<T> where
        T: SessionObject,
        T: 'static {
        pub fn new(object: Arc<RwLock<T>>) -> Self {
            SessionContainer {
                object
            }
        }
    }

    impl<T> Actor for SessionContainer<T> where
        T: SessionObject,
        T: 'static  {
        type Context = ws::WebsocketContext<Self>;
        fn started(&mut self, ctx: &mut Self::Context) {
            match self.object.write() {
                Ok(mut object) => object.session_started(ctx),
                Err(_) => ()
            }
        }
        fn stopped(&mut self, ctx: &mut Self::Context) {
            match self.object.write() {
                Ok(mut object) => object.session_stopped(ctx),
                Err(_) => ()
            }
        }
    }

    impl<T> StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionContainer<T> where
        T: SessionObject,
        T: 'static {
        fn handle(
            &mut self,
            msg: Result<ws::Message, ws::ProtocolError>,
            ctx: &mut Self::Context
        ) {
            match self.object.write() {
                Ok(mut object) => object.session_message(msg, ctx),
                Err(_) => ()
            }
        }
    }

    /// - GameController

    struct GameController {
        players_lobby: web::Data<RwLock<PlayersLobby>>,
        active_players:  Vec<Arc<RwLock<Player>>>,
        last_tick_start: Option<SystemTime>
    }
    impl GameController {
        pub fn new(players_lobby: web::Data<RwLock<PlayersLobby>>) -> Self {
            Self {
                players_lobby,
                active_players: Vec::new(),
                last_tick_start: None,
            }
        }
        fn delay_if_needed(&mut self) {
            const MINIMUM_DELAY_MILLIS: u64 = 150;
            match self.last_tick_start.and_then(|v| v.elapsed().ok()) {
                Some(difference) => {
                    let after_time = difference.as_millis() as u64;
                    if after_time < MINIMUM_DELAY_MILLIS {
                        let delay_time = MINIMUM_DELAY_MILLIS - after_time;
                        thread::sleep(Duration::from_millis(delay_time));
                    }
                },
                None => thread::sleep(Duration::from_millis(MINIMUM_DELAY_MILLIS)),
            }
            self.last_tick_start = Some(SystemTime::now());
        }
    }
    impl snake::game::GameController for GameController {
        fn game_action(&mut self) -> snake::game::ActionType {
            snake::game::ActionType::Start
        }
        fn game_start(&mut self) -> snake::world::Config {
            loop {
                let players = match self.game_lobby.read() {
                    Ok(game_lobby) => Some(game_lobby.players.clone()),
                    Err(_) => None,
                };
                match players {
                    Some(players) => if !players.is_empty() {
                        let snake_controllers = {
                            let mut controllers = HashMap::<usize, Rc<RefCell<dyn snake::world::SnakeController>>>::new();
                            let mut i: usize = 0;
                            for player in &players {
                                let controller = SnakeController::new(player.1.clone());
                                controllers.insert(i, Rc::new(RefCell::new(controller)));
                                i += 1;
                            }
                            controllers
                        };
                        self.players = Some(players);
                        return snake::world::Config {
                            world_size: (50, 50), /// Should be 255 to send as 8bit sized point!
                            eat_count: 3,
                            cut_tails: false,
                            base_snake_tail_size: 3,
                            snakes_controllers: snake_controllers,
                        }
                    },
                    None => {},
                }
                thread::sleep(Duration::from_secs(5));
            };
        }
        fn game_will_tick(
            &mut self,
            previous_world_view: &Option<WorldView>
        ) -> snake::game::TickType {
            self.delay_if_needed();
            match self.players {
                Some(_) => match previous_world_view {
                    Some(world_view) => if world_view.get_snakes_info().is_empty() {
                        snake::game::TickType::Break
                    } else {
                        snake::game::TickType::Common
                    },
                    None => snake::game::TickType::Initial,
                },
                None => snake::game::TickType::Break,
            }
        }
        fn game_did_tick(&mut self, world_view: &WorldView) {
            let map = world_view.get_world_mask().generate_map(|p| p.clone(), |o| o.clone());
            let mut buf = BytesMut::new();
            buf.put_u8(200); // Temp packet example!
            for (point, object) in map {
                buf.put_u8(point.x() as u8);
                buf.put_u8(point.y() as u8);
                buf.put_u8(match object {
                    ObjectType::Border => 0,
                    ObjectType::Snake(_) => 1,
                    ObjectType::Eat => 2,
                });
            }
            let bytes = buf.freeze();
            match &self.players {
                Some(players) => for (_, addr) in players {
                    addr.do_send(BinaryMessage(bytes.clone()));
                },
                None => {},
            }
        }
        fn game_end(&mut self, _: Result<(), CreateError>) {
            match &self.players {
                Some(players) => for (_, addr) in players {
                    addr.do_send(DirectionMessage {
                        with_reset: true
                    });
                },
                None => {},
            }
            self.last_tick_start = None;
            self.players = None;
        }
    }

    struct SnakeController {
        player: Arc<RwLock<Player>>
    }

    impl SnakeController {
        pub fn new(player: Arc<RwLock<Player>>) -> Self {
            SnakeController {
                player
            }
        }
    }

    impl snake::world::SnakeController for SnakeController {
        fn snake_will_burn(&mut self, _: &WorldView) {}
        fn snake_did_burn(&mut self, _: &SnakeInfo, _: &WorldView) {}
        fn snake_will_move(&mut self, _: &SnakeInfo, _: &WorldView) -> Direction {
            match self.player.read() {
                Ok(player) => match player.direction {
                    Some(direction) => direction,
                    None => Direction::Right
                },
                Err(_) => Direction::Right
            }
        }
        fn snake_did_move(&mut self, _: &SnakeInfo, _: &WorldView) {}
        fn snake_will_eat(&mut self, _: bool, _: &SnakeInfo, _: &WorldView) {}
        fn snake_did_eat(&mut self, _: bool, _: &SnakeInfo, _: &WorldView) {}
        fn snake_will_died(&mut self, _: &SnakeInfo, _: &WorldView) {}
        fn snake_did_died(&mut self, _: &WorldView) {}
    }

    pub async fn snake_game(players_lobby: web::Data<RwLock<PlayersLobby>>) {
        snake::game::Game::new(snake::game::Config {
            game_controller: Rc::new(RefCell::new(GameController::new(players_lobby))),
        }).unwrap().start();
    }
}

#[get("/snake")]
async fn snake(
    req: HttpRequest,
    stream: web::Payload,
    players_lobby: web::Data<RwLock<PlayersLobby>>
) -> Result<HttpResponse, Error> {
    let player = Arc::new(RwLock::new(Player::default()));
    match players_lobby.write() {
        Ok(mut players_lobby) => players_lobby.players.push(player.clone()),
        Err(_) => ()
    };
    ws::start(SessionContainer::new(player.clone()), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let players_lobby: web::Data<RwLock<PlayersLobby>> = web::Data::new(Default::default());

    Arbiter::spawn(snake_game(players_lobby.clone()));

    let app_factory = move || App::new()
        .app_data(players_lobby.clone())
        .service(snake);

    HttpServer::new(app_factory)
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
