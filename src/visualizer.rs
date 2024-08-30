use std::f64::consts::PI;

use js_sys::Array;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    network::{Level, NeuralNetwork},
    utils::{get_rgba, lerp},
};

pub fn draw_network(ctx: &CanvasRenderingContext2d, network: &NeuralNetwork) {
    let margin = 50.0;
    let left = margin;
    let top = margin;

    let Some(canvas) = ctx.canvas() else {
        panic!("Could not get canvas")
    };

    let width = canvas.width() as f64 - margin * 2.0;
    let height = canvas.height() as f64 - margin * 2.0;

    let level_height = height / network.levels.len() as f64;

    for i in (0..=network.levels.len() - 1).rev() {
        let level_top = top
            + lerp(
                height - level_height,
                0.0,
                if network.levels.len() == 1 {
                    0.5
                } else {
                    i as f64 / (network.levels.len() - 1) as f64
                },
            );

        {
            let line_dash: Array = Array::new();
            line_dash.push(&JsValue::from_f64(7.0));
            line_dash.push(&JsValue::from_f64(3.0));
            ctx.set_line_dash(&line_dash).unwrap();
        }

        draw_level(
            ctx,
            &network.levels[i],
            left,
            level_top,
            width,
            level_height,
            if i == network.levels.len() - 1 {
                vec!["⬆️", "⬅️", "➡️", "⬇️"]
            } else {
                vec![]
            },
        )
    }
}

pub fn draw_level(
    ctx: &CanvasRenderingContext2d,
    level: &Level,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    output_labels: Vec<&str>,
) {
    let right = left + width;
    let bottom = top + height;
    let Level {
        inputs,
        outputs,
        biases,
        weights,
    } = level;

    (0..inputs.len()).for_each(|i| {
        for j in 0..outputs.len() {
            ctx.begin_path();
            ctx.move_to(get_node_x(inputs, i as f64, left, right), bottom);
            ctx.line_to(get_node_x(outputs, j as f64, left, right), top);
            ctx.set_line_width(2.0);
            ctx.set_stroke_style(&JsValue::from_str(&get_rgba(weights[i][j])));
            ctx.stroke();
        }
    });

    let node_radius = 18;
    for i in 0..inputs.len() {
        let x = get_node_x(inputs, i as f64, left, right);
        ctx.begin_path();
        ctx.arc(x, bottom, node_radius as f64, 0.0, PI * 2.0)
            .unwrap();
        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill();
        ctx.begin_path();
        ctx.arc(x, bottom, node_radius as f64 * 0.6, 0.0, PI * 2.0)
            .unwrap();
        ctx.set_fill_style(&JsValue::from_str(&get_rgba(inputs[i])));
        ctx.fill();
    }

    for i in 0..outputs.len() {
        let x = get_node_x(outputs, i as f64, left, right);
        ctx.begin_path();
        ctx.arc(x, top, node_radius as f64, 0.0, PI * 2.0).unwrap();
        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill();
        ctx.begin_path();
        ctx.arc(x, top, node_radius as f64 * 0.6, 0.0, PI * 2.0)
            .unwrap();
        ctx.set_fill_style(&JsValue::from_str(&get_rgba(outputs[i])));
        ctx.fill();

        ctx.begin_path();
        ctx.set_line_width(2.0);
        ctx.arc(x, top, node_radius as f64 * 0.8, 0.0, PI * 2.0)
            .unwrap();
        ctx.set_stroke_style(&JsValue::from_str(&get_rgba(biases[i])));

        let line_dash = Array::new();
        line_dash.push(&JsValue::from_f64(3.0));
        line_dash.push(&JsValue::from_f64(3.0));

        ctx.set_line_dash(&line_dash).unwrap();
        ctx.stroke();
        ctx.set_line_dash(&Array::new()).unwrap();

        if let Some(output_label) = output_labels.get(i) {
            ctx.begin_path();
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_fill_style(&JsValue::from_str("black"));
            ctx.set_stroke_style(&JsValue::from_str("white"));
            ctx.set_font(&format!("{:.2}px Arial", node_radius as f64 * 1.5));
            ctx.fill_text(output_label, x, top + node_radius as f64 * 0.1)
                .unwrap();
            ctx.set_line_width(0.5);
            ctx.stroke_text(output_label, x, top + node_radius as f64 * 0.1)
                .unwrap();
        }
    }
}

pub fn get_node_x(nodes: &[f64], index: f64, left: f64, right: f64) -> f64 {
    lerp(
        left,
        right,
        if nodes.len() == 1 {
            0.5
        } else {
            index / (nodes.len() - 1) as f64
        },
    )
}
