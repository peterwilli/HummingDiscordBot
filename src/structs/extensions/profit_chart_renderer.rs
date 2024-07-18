use std::{
    sync::{atomic, Arc},
    thread::JoinHandle,
    time::Duration,
};

use anyhow::Result;
use headless_chrome::{protocol::cdp::Page, Browser, LaunchOptionsBuilder};
use log::debug;

use crate::structs::profit_chart;

pub trait ProfitChartRenderer {
    fn render_chart(&self) -> Result<Vec<u8>>;
}

impl<'c> ProfitChartRenderer for profit_chart::ChartData<'c> {
    fn render_chart(&self) -> Result<Vec<u8>> {
        let responder = move |r: tiny_http::Request| {
            let html = include_str!("../../../resources/balance_chart/dist/index.html");
            let response = tiny_http::Response::new(
                200.into(),
                vec![
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                ],
                std::io::Cursor::new(html),
                Some(html.len()),
                None,
            );
            r.respond(response)
        };

        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
        let shall_exit = Arc::new(atomic::AtomicBool::new(false));
        let srv = server.clone();
        let exit = shall_exit.clone();
        let _handler: JoinHandle<Result<(), std::io::Error>> = std::thread::spawn(move || {
            loop {
                if let Some(r) = srv.recv_timeout(Duration::from_millis(1000))? {
                    responder(r)?;
                }
                if exit.load(atomic::Ordering::Relaxed) {
                    break;
                }
            }
            Ok(())
        });

        let launch_opts = LaunchOptionsBuilder::default().headless(true).build()?;
        let browser = Browser::new(launch_opts)?;
        let tab = browser.new_tab()?;
        tab.set_transparent_background_color()?;
        let port = server.server_addr().to_ip().unwrap().port();
        let chart_json = serde_json::to_string(&self)?;
        debug!("chart_json: {}", chart_json);
        tab.navigate_to(&format!(
            "http://localhost:{}#{}",
            port,
            urlencoding::encode(&chart_json)
        ))?;
        tab.wait_until_navigated()?;
        let chart_screenshot = tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            None,
            Some(Page::Viewport {
                x: 0.0,
                y: 0.0,
                width: 450.0 + (15.0 * 2.0),
                height: 300.0 + (15.0 * 2.0),
                scale: 1.0,
            }),
            true,
        )?;
        shall_exit.store(true, atomic::Ordering::Relaxed);
        return Ok(chart_screenshot);
    }
}
