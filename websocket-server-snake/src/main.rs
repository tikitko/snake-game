use actix::{Actor, StreamHandler, AsyncContext, Addr, Running, ActorContext, Arbiter, ContextFutureSpawner, Handler, Message, MailboxError};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use std::thread;
use actix::clock::Duration;
use std::rc::Rc;
use actix_web::web::block;
use actix_web::rt::blocking::BlockingError;
use actix::prelude::Future;
use std::time::SystemTime;
use uuid::Uuid;
use websocket::{GameCoordinator, PlayerSession};
use std::cell::RefCell;

mod websocket {
    use std::collections::HashMap;
    use actix::{Addr, Actor, Running, Message, Handler, StreamHandler, ActorContext, AsyncContext};
    use actix_web::web;
    use std::sync::RwLock;
    use actix_web_actors::ws;
    use std::any::Any;
    use uuid::Uuid;
    use std::time::SystemTime;
    use actix::clock::Duration;
    use snake::Direction;
    use snake::world::{WorldView, SnakeInfo, CreateError};
    use std::thread;
    use std::rc::Rc;
    use std::cell::RefCell;
    use actix_web::web::{block, Bytes};

    pub struct GameCoordinator {
        players: HashMap<Uuid, Addr<PlayerSession>>
    }

    impl GameCoordinator {
        pub fn get_players(&self) -> &HashMap<Uuid, Addr<PlayerSession>> {
            &self.players
        }
    }

    impl Default for GameCoordinator {
        fn default() -> Self {
            GameCoordinator {
                players: HashMap::new(),
            }
        }
    }

    pub struct PlayerSession {
        id: Uuid,
        game_coordinator: web::Data<RwLock<GameCoordinator>>,
        direction: Option<Direction>,
    }

    impl PlayerSession {
        pub fn new(id: Uuid, game_coordinator: web::Data<RwLock<GameCoordinator>>) -> Self {
            PlayerSession {
                id,
                game_coordinator,
                direction: None,
            }
        }
    }

    impl Actor for PlayerSession {
        type Context = ws::WebsocketContext<Self>;
        fn started(&mut self, ctx: &mut Self::Context) {
            match self.game_coordinator.write() {
                Ok(mut game_coordinator) => {
                    game_coordinator.players.insert(self.id, ctx.address());
                },
                Err(_) => {},
            }
        }
        fn stopping(&mut self, _: &mut Self::Context) -> Running {
            match self.game_coordinator.write() {
                Ok(mut game_coordinator) => {
                    game_coordinator.players.remove(&self.id);
                }
                Err(_) => {}
            }
            Running::Stop
        }
    }
    pub struct DirectionMessage;
    impl Message for DirectionMessage {
        type Result = Option<Direction>;
    }
    impl Handler<DirectionMessage> for PlayerSession {
        type Result = Option<Direction>;

        fn handle(
            &mut self,
            _: DirectionMessage,
            _: &mut Self::Context
        ) -> Self::Result {
            self.direction
        }
    }

    pub struct BinaryMessage(Bytes);
    impl Message for BinaryMessage {
        type Result = ();
    }
    impl Handler<BinaryMessage> for PlayerSession {
        type Result = ();

        fn handle(
            &mut self,
            binary_message: BinaryMessage,
            ctx: &mut Self::Context
        ) -> Self::Result {
            ctx.binary(binary_message.0)
        }
    }

    impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlayerSession {
        fn handle(
            &mut self,
            msg: Result<ws::Message, ws::ProtocolError>,
            ctx: &mut Self::Context,
        ) {
            match msg {
                Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
                Ok(ws::Message::Text(text)) => ctx.text(text),
                Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
                Ok(ws::Message::Close(_)) => ctx.stop(),
                _ => (),
            }
        }
    }

    struct GameController {
        game_coordinator: web::Data<RwLock<GameCoordinator>>,
        last_tick_start: Option<SystemTime>,
        current_players: Option<HashMap<Uuid, Addr<PlayerSession>>>,
    }
    impl GameController {
        pub fn new(game_coordinator: web::Data<RwLock<GameCoordinator>>) -> Self {
            Self {
                game_coordinator,
                last_tick_start: None,
                current_players: None,
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
                let players = match self.game_coordinator.read() {
                    Ok(game_coordinator) => Some(game_coordinator.get_players().clone()),
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
                        self.current_players = Some(players);
                        return snake::world::Config {
                            world_size: (300, 300),
                            eat_count: 3,
                            cut_tails: true,
                            base_snake_tail_size: 3,
                            snakes_controllers: snake_controllers,
                        }
                    },
                    None => {},
                }
                thread::sleep(Duration::from_secs(5));
            };
        }
        fn game_will_tick(&mut self, previous_world_view: &Option<WorldView>) -> snake::game::TickType {
            self.delay_if_needed();
            match self.current_players {
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
            match &self.current_players {
                Some(players) => for (_, addr) in players {
                    let bytes = Bytes::from("PING");
                    addr.do_send(BinaryMessage(bytes));
                },
                None => {},
            }
        }
        fn game_end(&mut self, _: Result<(), CreateError>) {
            self.last_tick_start = None;
            self.current_players = None;
        }
    }
    struct SnakeController {
        addr: Addr<PlayerSession>,
    }

    impl SnakeController {
        pub fn new(addr: Addr<PlayerSession>) -> Self {
            Self {
                addr,
            }
        }
    }

    impl snake::world::SnakeController for SnakeController {
        fn snake_will_burn(&mut self, _: &WorldView) {}
        fn snake_did_burn(&mut self, _: &SnakeInfo, _: &WorldView) {}
        fn snake_will_move(&mut self, _: &SnakeInfo, _: &WorldView) -> Direction {
            let result = futures::executor::block_on(self.addr.send(DirectionMessage));
            match result.unwrap_or(None) {
                Some(direction) => direction,
                None => Direction::Right,
            }
        }
        fn snake_did_move(&mut self, _: &SnakeInfo, _: &WorldView) {}
        fn snake_will_eat(&mut self, _: bool, _: &SnakeInfo, _: &WorldView) {}
        fn snake_did_eat(&mut self, _: bool, _: &SnakeInfo, _: &WorldView) {}
        fn snake_will_died(&mut self, _: &SnakeInfo, _: &WorldView) {}
        fn snake_did_died(&mut self, _: &WorldView) {}
    }

    pub async fn snake_game(game_coordinator: web::Data<RwLock<GameCoordinator>>) {
        let game_controller = GameController::new(game_coordinator);
        snake::game::Game::new(snake::game::Config {
            game_controller: Rc::new(RefCell::new(game_controller)),
        }).unwrap().start();
    }
}

#[get("/snake")]
async fn snake(
    req: HttpRequest,
    stream: web::Payload,
    game_coordinator: web::Data<RwLock<websocket::GameCoordinator>>,
) -> Result<HttpResponse, Error> {
    let current_time_in_micros = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_micros(0))
        .as_micros();
    let id = Uuid::from_u128(current_time_in_micros);
    ws::start(websocket::PlayerSession::new(id, game_coordinator), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_coordinator: web::Data<RwLock<websocket::GameCoordinator>> = web::Data::new(Default::default());

    Arbiter::spawn(websocket::snake_game(game_coordinator.clone()));

    let app_factory = move || App::new()
        .app_data(game_coordinator.clone())
        .service(snake);

    HttpServer::new(app_factory)
        .bind("127.0.0.1:9000")?
        .run()
        .await
}