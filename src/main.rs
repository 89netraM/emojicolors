use image::GenericImageView;
use scraper::node::Element;
use std::fs::File;
use image::Rgb;
use image::Pixel;
use base64::decode;
use image::io::Reader as ImageReader;
use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};
use std::io::Cursor;
use std::collections::HashMap;
use serde::Serialize;

fn main() {
	let response = get("http://www.unicode.org/emoji/charts/full-emoji-list.html").unwrap();
	let html = response.text().unwrap();
	let document = Html::parse_document(&html);
	let selector = Selector::parse("tr > td:nth-child(8) > img").unwrap();
	let map: HashMap<_, _> = document.select(&selector).filter_map(make_kvp).collect();
	let file = File::create("./emojicolors.json").unwrap();
	serde_json::to_writer(file, &map).unwrap();
}

fn make_kvp(elem_ref: ElementRef) -> Option<(String, Info)> {
	let elem = elem_ref.value();
	Some((
		elem.attr("alt")?.to_string(),
		Info::from_elem(elem)?
	))
}

#[derive(Debug, Serialize)]
struct Info {
	pub average: Vec<u8>,
	pub primary: Vec<u8>,
	pub secondary: Vec<u8>,
}

impl Info {
	pub fn from_elem(elem: &Element) -> Option<Info> {
		let base64 = elem.attr("src")?.split(",").skip(1).next()?;
		let image = ImageReader::new(Cursor::new(decode(base64).ok()?))
			.with_guessed_format()
			.expect("No reader!")
			.decode()
			.expect("No decode!");

		let mut total_pixels = 0.0;
		let mut average = *Rgb::from_slice(&[0.0, 0.0, 0.0]);
		let mut pixel_count: HashMap<_, u32> = HashMap::new();
		for (_x, _y, pixel_rgba) in image.pixels() {
			if pixel_rgba[3] != 0 {
				let pixel = Rgb::from_slice(pixel_rgba.channels().get(0..3)?);
				total_pixels += 1.0;
				average[0] += pixel[0] as f32;
				average[1] += pixel[1] as f32;
				average[2] += pixel[2] as f32;

				*pixel_count.entry(*pixel).or_insert(0) += 1;
			}
		}
		average[0] = average[0] / total_pixels;
		average[1] = average[1] / total_pixels;
		average[2] = average[2] / total_pixels;

		let mut primary = None;
		let mut primary_count = 0;
		let mut secondary = None;
		let mut secondary_count = 0;
		for (pixel, count) in pixel_count {
			if count > primary_count {
				primary = Some(pixel);
				primary_count = count;
			}
			else if count > secondary_count {
				secondary = Some(pixel);
				secondary_count = count;
			}
		}

		Some(Info {
			average: average.channels().iter().map(|f| *f as u8).collect(),
			primary: primary?.channels().iter().map(|c| *c).collect(),
			secondary: secondary?.channels().iter().map(|c| *c).collect(),
		})
	}
}
