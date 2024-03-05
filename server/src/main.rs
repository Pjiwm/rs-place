use actix_web::{
    get, post,
    web::{Data, Json, Path},
    App, HttpResponse, HttpServer, Responder,
};
use serde::ser::Error;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use serde_derive::Deserialize;
use std::{
    io,
    sync::atomic::{AtomicU32, Ordering},
};

const X_MAX: u32 = 1000;
const Y_MAX: u32 = 1000;
const GRID_SIZE: usize = 4;
type State = Data<Vec<Grid>>;

#[derive(serde_derive::Serialize, Deserialize)]
struct Grid {
    width: u32,
    height: u32,
    cells: Vec<AtomicU32>,
}

impl Grid {
    fn get_cell_ref(&self, x: u32, y: u32) -> Option<&AtomicU32> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(&self.cells[index as usize])
        } else {
            None
        }
    }
    fn to_2d(&self) -> Vec<Vec<u32>> {
        self.cells
            .chunks(self.width as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|atomic| atomic.load(Ordering::Relaxed))
                    .collect()
            })
            .collect()
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let grids = (0..GRID_SIZE)
        .into_iter()
        .map(|_| {
            let cells = (0..X_MAX * Y_MAX)
                .map(|_| AtomicU32::new(0xFF_FF_FF))
                .collect();
            Grid {
                width: X_MAX,
                height: Y_MAX,
                cells,
            }
        })
        .collect();

    let state: State = Data::new(grids);
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(send)
            .service(get_state)
            .service(ping)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[derive(Deserialize, Clone, Debug)]
struct Pos {
    x: u32,
    y: u32,
    rgba: u32,
}

impl Serialize for Pos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.x > X_MAX || self.y > Y_MAX {
            return Err(S::Error::custom(format!(
                "Invalid position, positions should be between 0 and {}x, {}y, but were, {}x and {}y",
                X_MAX, Y_MAX, self.x, self.y
            )));
        }

        let mut state = serializer.serialize_struct("Pos", 3)?;

        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        let hex = format!("{:06X}", self.rgba);
        state.serialize_field("rgbs", &hex)?;
        state.end()
    }
}

#[get("/state/{grid}")]
async fn get_state(data: State, path: Path<usize>) -> impl Responder {
    let grid_idx = path.into_inner();
    if let Some(grid) = data.get(grid_idx) {
        HttpResponse::Ok().json(grid.to_2d())
    } else {
        HttpResponse::BadRequest().json(format!(
            "grid {grid_idx} does not exist, total grid size is: {GRID_SIZE}"
        ))
    }
}

#[post("/send/{grid}")]
async fn send(req_body: Json<Pos>, data: State, path: Path<usize>) -> impl Responder {
    let grid_idx = path.into_inner();
    let grid = match data.get(grid_idx) {
        Some(o) => o,
        None => {
            return HttpResponse::BadRequest().json(format!(
                "grid {grid_idx} does not exist, total grid size is: {GRID_SIZE}"
            ))
        }
    };
    let pos = req_body.into_inner();
    if let Some(atomic) = grid.get_cell_ref(pos.x, pos.y) {
        atomic.store(pos.rgba, Ordering::Relaxed);
        HttpResponse::Ok().json("Success")
    } else {
        HttpResponse::BadRequest().json("Server could not set new color.")
    }
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json("pong!")
}
