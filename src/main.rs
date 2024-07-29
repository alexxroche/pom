//use chrono::{DateTime, Utc};
use chrono::{DateTime, Local};
use pom::phasehunt;

fn main() {
    /*
      Based on:
        # pom ver 0.2 20180516003029 alexx at alexx dot net (added module checker)
        # pom ver 0.1 20070104 alexx at alexx dot net, MIT Licence
       both of which were a thin wrapper to the perl module Astro::MoonPhase;
    */

    //let p: Vec<String> = phasehunt(Some(Utc::now().timestamp() as f64), None)
    let dt = Local::now();
    let offset = dt.offset().clone();
    //let p: Vec<String> = phasehunt(None, Some(offset.local_minus_utc() as i32))
    let p: Vec<String> = phasehunt(None, None)
        .into_iter()
        .map(|x| DateTime::from_timestamp(x as i64, 0).unwrap().naive_local())
        .map(|y| {
            DateTime::<Local>::from_naive_utc_and_offset(y, offset)
                .format("%a %b %e %H:%M:%S %Y (%Z)")
                .to_string()
        })
        .collect();

    println!("New noon      = {}", p[0]);
    println!("First quarter = {}", p[1]);
    println!("Full Moon     = {}", p[2]);
    println!("Last quarter  = {}", p[3]);
    println!("New moon      = {}", p[4]);
}
