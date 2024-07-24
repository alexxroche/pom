/*
  A direct port of the perl module Astro::MoonPhase;
*/

use chrono::Utc;

// Astronomical constants.
/*
#[allow(dead_code)]
const EPOCH: f64 = 2444238.5; // 1980 january 0.0

// Constants defining the Sun's apparent orbit.

#[allow(dead_code)]
const ELONGE: f64 = 278.833540; // ecliptic longitude of the Sun at epoch 1980.0
#[allow(dead_code)]
const ELONGP: f64 = 282.596403; // ecliptic longitude of the Sun at perigee
#[allow(dead_code)]
const ECCENT: f64 = 0.016718; // eccentricity of Earth's orbit
#[allow(dead_code)]
const SUNSMAX: f64 = 1.495985e8; // semi-major axis of Earth's orbit, km
#[allow(dead_code)]
const SUNANGSIZ: f64 = 0.533128; // sun's angular size, degrees, at semi-major axis distance

// Elements of the Moon's orbit, epoch 1980.0.

#[allow(dead_code)]
const MMLONG: f64 = 64.975464; // moon's mean longitude at the epoch
#[allow(dead_code)]
const MMLONGP: f64 = 349.383063; // mean longitude of the perigee at the epoch
#[allow(dead_code)]
const MLNODE: f64 = 151.950429; // mean longitude of the node at the epoch
#[allow(dead_code)]
const MINC: f64 = 5.145396; // inclination of the Moon's orbit
#[allow(dead_code)]
const MECC: f64 = 0.054900; // eccentricity of the Moon's orbit
#[allow(dead_code)]
const MANGSIZ: f64 = 0.5181; // moon's angular size at distance a from Earth
#[allow(dead_code)]
const MSMAX: f64 = 384401.0; // semi-major axis of Moon's orbit in km
#[allow(dead_code)]
const MPARALLAX: f64 = 0.9507; // parallax at distance a from Earth
*/

const SYNMONTH: f64 = 29.53058868; // synodic month (new Moon to new Moon)

// Properties of the Earth.

static PI: f64 = 3.14159265358979323846; // assume not near black hole nor in Tennessee

// Handy mathematical functions.

fn torad(d: f64) -> f64 {
    d * (PI / 180.0)
} // deg->rad

fn dsin(d: f64) -> f64 {
    torad(d).sin()
} // sin from deg

fn dcos(d: f64) -> f64 {
    torad(d).cos()
} // cos from deg

// libm::atan2 already has asin and atan

// jtime - convert internal date and time to astronomical Julian
// time (i.e. Julian date plus day fraction)

fn jtime(t: f64) -> f64 {
    (t / 86400.0) + 2440587.5 // (seconds /(seconds per day)) + julian date of epoch
}

// jdaytosecs - convert Julian date to a UNIX epoch

#[allow(dead_code)]
fn jdaytosecs(jday: f64) -> f64 {
    (jday - 2440587.5) * 86400.0 // (juliandate - jdate of unix epoch)*(seconds per julian day)
}

// jyear - convert Julian date to year, month, day, which are
// returned via integer pointers to integers
fn jyear(td: f64, yy: &mut f64, mm: &mut f64, dd: &mut f64) -> () {
    let td: f64 = td + 0.5; // astronomical to civil
    let z: f64 = td.floor();
    let f: f64 = td - z;

    let a: f64 = if z < 2299161.0 {
        z
    } else {
        let alpha = ((z - 1867216.25) / 36524.25).floor();
        z + 1.0 + alpha - (alpha / 4.0).floor()
    };

    let b: f64 = a + 1524.0;
    let c: f64 = ((b - 122.1) / 365.25).floor();
    let d: f64 = (365.25 * c).floor();
    let e: f64 = ((b - d) / 30.6001).floor();

    *dd = b - d - (30.6001 * e).floor() + f;
    if e < 14.0 {
        *mm = e - 1.0;
    } else {
        *mm = e - 13.0;
    };
    if *mm > 2.0 {
        *yy = c - 4716.0;
    } else {
        *yy = c - 4715.0;
    };
}

////  meanphase  --  Calculates  time  of  the mean new Moon for a given
////                 base date.  This argument K to this function is the
////                 precomputed synodic month index, given by:
////
////                        K = (year - 1900) * 12.3685
////
////                 where year is expressed as a year and fractional year.

fn meanphase(sdate: f64, k: f64) -> f64 {
    //// Time in Julian centuries from 1900 January 0.5
    let t: f64 = (sdate - 2415020.0) / 36525.0;
    let t2: f64 = t * t; // Square for frequent use
    let t3: f64 = t2 * t; // Cube for frequent use

    let nt1: f64 = 2415020.75933 + SYNMONTH * k + 0.0001178 * t2 - 0.000000155 * t3
        + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);
    nt1
}

// truephase - given a K value used to determine the mean phase of the
// new moon, and a phase selector (0.0, 0.25, 0.5, 0.75),
// obtain the true, corrected phase time

fn truephase(k: f64, phase: f64) -> f64 {
    let mut apcor = 0.0;

    let k = k + phase; // add phase to new moon time
    let t = k / 1236.85; // time in Julian centuries from
                         // 1900 January 0.5
    let t2 = t * t; // square for frequent use
    let t3 = t2 * t; // cube for frequent use

    // mean time of phase
    let mut pt = 2415020.75933 + SYNMONTH * k + 0.0001178 * t2 - 0.000000155 * t3
        + 0.00033 * dsin(166.56 + 132.87 * t - 0.009173 * t2);

    // Sun's mean anomaly
    let m = 359.2242 + 29.10535608 * k - 0.0000333 * t2 - 0.00000347 * t3;

    // Moon's mean anomaly
    let mprime = 306.0253 + 385.81691806 * k + 0.0107306 * t2 + 0.00001236 * t3;

    // Moon's argument of latitude
    let f = 21.2964 + 390.67050646 * k - 0.0016528 * t2 - 0.00000239 * t3;

    if phase < 0.01 || (phase - 0.5).abs() < 0.01 {
        // Corrections for New and Full Moon.

        pt += (0.1734 - 0.000393 * t) * dsin(m) + 0.0021 * dsin(2.0 * m) - 0.4068 * dsin(mprime)
            + 0.0161 * dsin(2.0 * mprime)
            - 0.0004 * dsin(3.0 * mprime)
            + 0.0104 * dsin(2.0 * f)
            - 0.0051 * dsin(m + mprime)
            - 0.0074 * dsin(m - mprime)
            + 0.0004 * dsin(2.0 * f + m)
            - 0.0004 * dsin(2.0 * f - m)
            - 0.0006 * dsin(2.0 * f + mprime)
            + 0.0010 * dsin(2.0 * f - mprime)
            + 0.0005 * dsin(m + 2.0 * mprime);
        apcor = 1.0;
    } else if (phase - 0.25).abs() < 0.01 || (phase - 0.75).abs() < 0.01 {
        pt += (0.1721 - 0.0004 * t) * dsin(m) + 0.0021 * dsin(2.0 * m) - 0.6280 * dsin(mprime)
            + 0.0089 * dsin(2.0 * mprime)
            - 0.0004 * dsin(3.0 * mprime)
            + 0.0079 * dsin(2.0 * f)
            - 0.0119 * dsin(m + mprime)
            - 0.0047 * dsin(m - mprime)
            + 0.0003 * dsin(2.0 * f + m)
            - 0.0004 * dsin(2.0 * f - m)
            - 0.0006 * dsin(2.0 * f + mprime)
            + 0.0021 * dsin(2.0 * f - mprime)
            + 0.0003 * dsin(m + 2.0 * mprime)
            + 0.0004 * dsin(m - 2.0 * mprime)
            - 0.0003 * dsin(2.0 * m + mprime);
        if phase < 0.5 {
            // First quarter correction.
            pt += 0.0028 - 0.0004 * dcos(m) + 0.0003 * dcos(mprime);
        } else {
            // Last quarter correction.
            pt += -0.0028 + 0.0004 * dcos(m) - 0.0003 * dcos(mprime);
        }
        apcor = 1.0;
    }
    //asset!(apcor, format!("truephase() called with invalid phase selector ({}).\n", phase);
    assert_eq!(
        apcor, 1.0,
        "{}", format!("truephase() called with invalid phase selector ({}).\n", phase)
    );
    pt
}

// phasehunt - find time of phases of the moon which surround the current
// date.  Five phases are found, starting and ending with the
// new moons which bound the current lunation

//fn phasehunt<Tz: chrono::TimeZone>(sdate: Option<DateTime<Tz>>) -> Vec<f64> {
pub fn phasehunt(sdate: Option<f64>) -> Vec<f64> {
    let sdate: f64 = match sdate {
        None => jtime(Utc::now().timestamp() as f64),
        Some(_) => jtime(sdate.expect("Try converting to a timestamp")),
    };

    let mut adate = sdate - 45.0;
    let mut yy = 0.0;
    let mut mm = 0.0;
    let mut dd = 0.0;

    jyear(adate, &mut yy, &mut mm, &mut dd);
    let mut k1: f64 = ((yy + ((mm - 1.0) * (1.0 / 12.0)) - 1900.0) * 12.3685).floor();

    let nt1 = meanphase(adate, k1);
    adate = nt1;

    let mut nt1 = 0.0;
    let mut k2: f64;

    loop {
        adate += SYNMONTH;
        k2 = k1 + 1.0;
        let nt2 = meanphase(adate, k2);
        if nt1 <= sdate && nt2 > sdate {
            break;
        }
        nt1 = nt2;
        k1 = k2;
    }

    Vec::from([
        jdaytosecs(truephase(k1, 0.0)),
        jdaytosecs(truephase(k1, 0.25)),
        jdaytosecs(truephase(k1, 0.5)),
        jdaytosecs(truephase(k1, 0.75)),
        jdaytosecs(truephase(k2, 0.0)),
    ])
}

/*
=head1 ABOUT THE ALGORITHMS

The algorithms used in this program to calculate the positions of Sun and
Moon as seen from the Earth are given in the book I<Practical Astronomy
With  Your  Calculator>  by  B<Peter  Duffett-Smith,   Second   Edition,
Cambridge University Press, 1981>.  Ignore the word "Calculator" in the
title;  this  is  an  essential  reference  if  you're  interested  in
developing  software  which  calculates  planetary  positions, orbits,
eclipses, and  the  like.   If  you're  interested  in  pursuing  such
programming, you should also obtain:

I<Astronomical  Formulae for Calculators> by B<Jean Meeus, Third Edition,
Willmann-Bell, 1985>.  A must-have.

I<Planetary  Programs  and  Tables  from  -4000  to  +2800>  by  B<Pierre
Bretagnon  and Jean-Louis Simon, Willmann-Bell, 1986>.  If you want the
utmost  (outside  of  JPL)  accuracy  for  the  planets,  it's   here.

I<Celestial BASIC> by B<Eric Burgess, Revised Edition, Sybex, 1985>.  Very
cookbook oriented, and many of the algorithms are hard to dig  out  of
the turgid BASIC code, but you'll probably want it anyway.

Many of these references can be obtained from Willmann-Bell, P.O.  Box
35025,  Richmond,  VA 23235, USA.  Phone: (804) 320-7016.  In addition
to their own publications, they stock most of the standard  references
for mathematical and positional astronomy.

=head1 LICENCE

This  program is in the public domain: "Do what thou wilt shall be the
whole of the law".

=head1 AUTHORS

The moontool.c Release 2.0:

    A Moon for the Sun
    Designed and implemented by John Walker in December 1987,
    revised and updated in February of 1988.

Initial Perl transcription:

    Raino Pikkarainen, 1998
    raino.pikkarainen@saunalahti.fi

The moontool.c Release 2.4:

    Major enhancements by Ron Hitchens, 1989

Revisions:

    Brett Hamilton  http://simple.be/
    Bug fix, 2003
    Second transcription and bugfixes, 2004

    Christopher J. Madsen  http://www.cjmweb.net/
    Added phaselist function, March 2007
*/
