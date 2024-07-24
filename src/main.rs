//use chrono::{DateTime, Utc};
use chrono::DateTime;
use pom::phasehunt;

fn main() {
    /*
      Based on:
        # pom ver 0.2 20180516003029 alexx at alexx dot net (added module checker)
        # pom ver 0.1 20070104 alexx at alexx dot net, MIT Licence
       both of which were a thin wrapper to the perl module Astro::MoonPhase;
    */

    //let p: Vec<String> = phasehunt(Some(Utc::now().timestamp() as f64))
    let p: Vec<String> = phasehunt(None)
        .into_iter()
        .map(|x| {
            DateTime::from_timestamp(x as i64, 0)
                .unwrap()
                .format("%a %b %d %H:%M:%S %Y")
                .to_string()
        })
        .collect();

    println!("New noon      = {}", p[0]);
    println!("First quarter = {}", p[1]);
    println!("Full Moon     = {}", p[2]);
    println!("Last quarter  = {}", p[3]);
    println!("New moon      = {}", p[4]);
}
