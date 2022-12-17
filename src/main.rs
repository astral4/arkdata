#![warn(clippy::all, clippy::pedantic)]

use arkdata::Details;

fn main() {
    let _details = Details::get(None);
}
