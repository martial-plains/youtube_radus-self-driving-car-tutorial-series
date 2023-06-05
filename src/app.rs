use gloo::{
    render::{request_animation_frame, AnimationFrame},
    utils::{document, format::JsValueSerdeExt, window},
};
use js_sys::JSON;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Storage};
use yew::{html, Callback, Component};

use crate::{
    car::{Car, CarPtr},
    controls::ControlKind,
    road::Road,
    visualizer,
};

#[derive(Debug, Default)]
pub enum Msg {
    #[default]
    None,
    Save,
    Discard,
    AnimationFrame(f64),
}

#[derive(Debug, Default)]
pub struct App {
    network_canvas: Option<HtmlCanvasElement>,
    network_ctx: Option<CanvasRenderingContext2d>,
    car_canvas: Option<HtmlCanvasElement>,
    car_ctx: Option<CanvasRenderingContext2d>,
    traffic: Vec<CarPtr>,
    road: Road,
    animation: Option<AnimationFrame>,

    cars: Vec<CarPtr>,
    best_car: Option<CarPtr>,
    storage: Option<Storage>,
}

impl App {
    fn generate_cars(&self, amount: usize) -> Vec<CarPtr> {
        let mut cars = Vec::new();
        for _ in 1..=amount {
            cars.push(Car::new(
                self.road.get_late_center(1),
                100.0,
                30.0,
                50.0,
                ControlKind::AI,
                None,
            ));
        }

        cars
    }

    pub fn save(&self) {
        if let Some(best_car) = &self.best_car {
            self.storage.as_ref().map(|storage| {
                storage
                    .set_item(
                        "best_brain",
                        JSON::stringify(
                            &<::wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(
                                &best_car.brain,
                            )
                            .unwrap(),
                        )
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .as_str(),
                    )
                    .unwrap();
                Some(())
            });
        }
    }

    pub fn discard(&mut self) {
        if let Some(storage) = &self.storage {
            storage.remove_item("best_brain").unwrap();
        }
    }

    fn animate(&mut self, time: f64) {
        let car_canvas = match &self.car_canvas {
            Some(value) => value,
            None => return,
        };

        let network_canvas = match &self.network_canvas {
            Some(value) => value,
            None => return,
        };

        let car_ctx = match &self.car_ctx {
            Some(value) => value,
            None => return,
        };

        let network_ctx = match &self.network_ctx {
            Some(value) => value,
            None => return,
        };

        for i in 0..self.traffic.len() {
            self.traffic[i].update(&self.road.borders, &Vec::new());
        }

        for i in 0..self.cars.len() {
            self.cars[i].update(&self.road.borders, &self.traffic);
        }

        self.best_car = Some(
            self.cars
                .iter()
                .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
                .unwrap()
                .clone(),
        );

        car_canvas.set_height(window().inner_height().unwrap().as_f64().unwrap() as u32);
        network_canvas.set_height(window().inner_height().unwrap().as_f64().unwrap() as u32);

        car_ctx.save();
        if let Some(best_car) = &self.best_car {
            car_ctx
                .translate(0.0, -best_car.y + car_canvas.height() as f64 * 0.7)
                .unwrap()
        }

        self.road.draw(car_ctx);

        for i in 0..self.traffic.len() {
            self.traffic[i].draw(car_ctx, "red");
        }

        car_ctx.set_global_alpha(0.2);

        for i in 0..self.cars.len() {
            self.cars[i].draw(car_ctx, "blue");
        }

        car_ctx.set_global_alpha(1.0);

        if let Some(best_car) = &mut self.best_car {
            best_car.draw(car_ctx, "blue")
        }

        car_ctx.restore();

        network_ctx.set_line_dash_offset(-time / 50.0);
        if let Some(best_car) = &self.best_car {
            if let Some(brain) = &best_car.brain {
                visualizer::draw_network(network_ctx, brain);
            }
        }
    }
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(_: &yew::Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let save_button_onclick = {
            let link = ctx.link().clone();
            Callback::from(move |_| {
                link.send_message(Msg::Save);
            })
        };
        let discard_button_onclick = {
            let link = ctx.link().clone();
            Callback::from(move |_| {
                link.send_message(Msg::Discard);
            })
        };

        html! {
            <>
                <canvas id="carCanvas"></canvas>
                <div id="verticalButtons">
                    <button id="saveButton" onclick={save_button_onclick}>{"💾"}</button>
                        <button id="discardButton" onclick={discard_button_onclick}>{"🗑️"}</button>
                </div>
                <canvas id="networkCanvas"></canvas>
            </>
        }
    }

    fn update(&mut self, _: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::None => (),
            Msg::Save => self.save(),
            Msg::Discard => self.discard(),
            Msg::AnimationFrame(time) => self.animate(time),
        }
        true
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
        if first_render {
            self.car_canvas = Some(
                document()
                    .get_element_by_id("carCanvas")
                    .unwrap()
                    .dyn_into::<HtmlCanvasElement>()
                    .unwrap(),
            );

            if let Some(car_canvas) = &self.car_canvas {
                car_canvas.set_width(200);
                self.car_ctx = Some(
                    car_canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<CanvasRenderingContext2d>()
                        .unwrap(),
                );

                self.road = Road::new(
                    car_canvas.width() as f64 / 2.0,
                    car_canvas.width() as f64 * 0.9,
                    None,
                );
            }

            self.network_canvas = Some(
                document()
                    .get_element_by_id("networkCanvas")
                    .unwrap()
                    .dyn_into::<HtmlCanvasElement>()
                    .unwrap(),
            );

            if let Some(network_canvas) = self.network_canvas.as_ref() {
                network_canvas.set_width(300);
                self.network_ctx = Some(
                    network_canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<CanvasRenderingContext2d>()
                        .unwrap(),
                );
            }

            self.cars = self.generate_cars(1);

            self.traffic = vec![
                Car::new(
                    self.road.get_late_center(1),
                    -100.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
                Car::new(
                    self.road.get_late_center(0),
                    -300.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
                Car::new(
                    self.road.get_late_center(2),
                    -300.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
                Car::new(
                    self.road.get_late_center(0),
                    -500.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
                Car::new(
                    self.road.get_late_center(1),
                    -500.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
                Car::new(
                    self.road.get_late_center(1),
                    -700.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
                Car::new(
                    self.road.get_late_center(2),
                    -700.0,
                    30.0,
                    50.0,
                    ControlKind::Dummy,
                    Some(2.0),
                ),
            ];

            self.storage = web_sys::window().unwrap().local_storage().unwrap();

            if let Some(storage) = &self.storage {
                if let Ok(Some(item)) = storage.get_item("best_brain") {
                    for i in 0..self.cars.len() {
                        self.cars[i].brain = gloo::utils::format::JsValueSerdeExt::into_serde(
                            &JSON::parse(&item).unwrap(),
                        )
                        .unwrap()
                    }
                }
            }
        }

        {
            let link = ctx.link().clone();
            self.animation = Some(request_animation_frame(move |timestamp| {
                link.send_message(Msg::AnimationFrame(timestamp));
            }));
        }
    }
}
