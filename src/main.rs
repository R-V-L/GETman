use eframe::egui;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 580.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GETman 0.1",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_pixels_per_point(2.0);
            Ok(Box::new(HttpClientApp::default()))
        }),
    )
}

struct HttpClientApp {
    url: String,
    method: String,
    headers: String,
    body: String,
    response: String,
    client: Client,
}

impl Default for HttpClientApp {
    fn default() -> Self {
        Self {
            url: "https://httpbin.org/get".to_string(),
            method: "GET".to_string(),
            headers: String::new(),
            body: String::new(),
            response: String::new(),
            client: Client::new(),
        }
    }
}

impl eframe::App for HttpClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.add(egui::TextEdit::singleline(&mut self.url).desired_width(f32::INFINITY));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Method:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.method.clone())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.method, "GET".to_string(), "GET");
                            ui.selectable_value(&mut self.method, "POST".to_string(), "POST");
                            ui.selectable_value(&mut self.method, "PUT".to_string(), "PUT");
                            ui.selectable_value(&mut self.method, "DELETE".to_string(), "DELETE");
                            ui.selectable_value(&mut self.method, "PATCH".to_string(), "PATCH");
                        });
                    
                    if ui.button("Send Request").clicked() {
                        let future = self.send_request();
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        self.response = rt.block_on(future);
                    }
                });
                
                let available_height = ui.available_height();
                let text_edit_height = (available_height - 80.0) / 2.0;
                let response_box_height = available_height - 45.0;

                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        ui.label("Headers (Key: Value, one per line):");
                        ui.add_sized(
                            [ui.available_width(), text_edit_height],
                            egui::TextEdit::multiline(&mut self.headers),
                        );
                        
                        ui.label("Body (optional):");
                        ui.add_sized(
                            [ui.available_width(), text_edit_height],
                            egui::TextEdit::multiline(&mut self.body),
                        );
                    });

                    columns[1].vertical(|ui| {
                        ui.label("Response:");
                        egui::ScrollArea::vertical().max_height(response_box_height).show(ui, |ui| {
                            ui.add_sized(
                                [ui.available_width(), response_box_height],
                                egui::TextEdit::multiline(&mut self.response),
                            );
                        });
                    });
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    ui.label("Created by: R-V-L");
                });
            });
        });
    }
}

impl HttpClientApp {
    async fn send_request(&self) -> String {
        let mut request = match self.method.as_str() {
            "GET" => self.client.get(&self.url),
            "POST" => self.client.post(&self.url),
            "PUT" => self.client.put(&self.url),
            "DELETE" => self.client.delete(&self.url),
            "PATCH" => self.client.patch(&self.url),
            _ => return "Invalid HTTP method".to_string(),
        };

        let headers: HashMap<_, _> = self.headers
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].trim(), parts[1].trim()))
                } else {
                    None
                }
            })
            .collect();

        for (key, value) in headers {
            request = request.header(key, value);
        }

        if !self.body.is_empty() {
            request = request.body(self.body.clone());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();
                let body = response.text().await.unwrap_or_else(|e| e.to_string());

                // Format the response
                let mut formatted_response = format!("Status: {}\n\nHeaders:\n", status);
                for (key, value) in headers.iter() {
                    formatted_response.push_str(&format!("{}: {}\n", key, value.to_str().unwrap_or("Invalid header value")));
                }
                formatted_response.push_str("\nBody:\n");

                // Try to parse and pretty print JSON
                match serde_json::from_str::<Value>(&body) {
                    Ok(json) => formatted_response.push_str(&serde_json::to_string_pretty(&json).unwrap()),
                    Err(_) => formatted_response.push_str(&body),
                }

                formatted_response
            }
            Err(e) => format!("Error: {}", e),
        }
    }
}