/*
  A direct port of the perl module Astro::MoonPhase;
*/

use chrono::{offset::FixedOffset, Local, Utc};

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

//static PI: f64 = 3.141_592_653_589_793; // assume not near black hole nor in Tennessee
static PI: f64 = std::f64::consts::PI; // assume not near black hole nor in Tennessee

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

fn jdaytosecs(jday: f64) -> f64 {
    (jday - 2440587.5) * 86400.0 // (juliandate - jdate of unix epoch)*(seconds per julian day)
}

// jyear - convert Julian date to year, month, day, which are
// returned via integer pointers to integers
fn jyear(td: f64, yy: &mut f64, mm: &mut f64, dd: &mut f64) {
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

/*  meanphase  --  Calculates  time  of  the mean new Moon for a given
                   base date.  This argument K to this function is the
                   precomputed synodic month index, given by:

                          K = (year - 1900) * 12.3685

                   where year is expressed as a year and fractional year.
*/

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
        "truephase() called with invalid phase selector ({}).\n",
        phase
    );
    pt
}

// phasehunt - find time of phases of the moon which surround the current
// date.  Five phases are found, starting and ending with the
// new moons which bound the current lunation

//fn phasehunt<Tz: chrono::TimeZone>(sdate: Option<DateTime<Tz>>) -> Vec<f64> {
pub fn phasehunt(sdate: Option<f64>, tz: Option<i32>) -> Vec<f64> {
    let sdate: f64 = match sdate {
        None => match tz {
            None => jtime(Local::now().timestamp() as f64),
            Some(tmz) => jtime(
                Utc::now()
                    .with_timezone(&{ FixedOffset::east_opt(tmz * 60i32 * 60i32).unwrap() })
                    .timestamp() as f64,
            ),
        },
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
package Astro::MoonPhase;
@EXPORT = qw(phase phasehunt phaselist);
$VERSION = '0.60';

# Handy mathematical functions.

sub sgn		{ return (($_[0] < 0) ? -1 : ($_[0] > 0 ? 1 : 0)); } 	# extract sign
sub fixangle	{ return ($_[0] - 360.0 * (floor($_[0] / 360.0))); }	# fix angle
sub torad	{ return ($_[0] * ($Pi / 180.0)); }						# deg->rad
sub todeg	{ return ($_[0] * (180.0 / $Pi)); }						# rad->deg
sub dsin	{ return (sin(torad($_[0]))); }						# sin from deg
sub dcos	{ return (cos(torad($_[0]))); }						# cos from deg

sub tan		{ return sin($_[0])/cos($_[0]); }
sub asin	{ return ($_[0]<-1 or $_[0]>1) ? undef : atan2($_[0],sqrt(1-$_[0]*$_[0])); }
sub atan {
    if		($_[0]==0)	{ return 0; }
    elsif	($_[0]>0)	{ return atan2(sqrt(1+$_[0]*$_[0]),sqrt(1+1/($_[0]*$_[0]))); }
    else 				{ return -atan2(sqrt(1+$_[0]*$_[0]),sqrt(1+1/($_[0]*$_[0]))); }
}

sub floor {
  my $val   = shift;
  my $neg   = $val < 0;
  my $asint = int($val);
  my $exact = $val == $asint;

  return ($exact ? $asint : $neg ? $asint - 1 : $asint);
}

# phaselist - find time of phases of the moon between two dates
# times (in & out) are seconds_since_1970

sub phaselist
{
  my ($sdate, $edate) = map { jtime($_) } @_;

  my (@phases, $d, $k, $yy, $mm);

  jyear($sdate, \$yy, \$mm, \$d);
  $k = floor(($yy + (($mm - 1) * (1.0 / 12.0)) - 1900) * 12.3685) - 2;

  while (1) {
    ++$k;
    for my $phase (0.0, 0.25, 0.5, 0.75) {
      $d = truephase($k, $phase);

      return @phases if $d >= $edate;

      if ($d >= $sdate) {
        push @phases, int(4 * $phase) unless @phases;
        push @phases, jdaytosecs($d);
      } # end if date should be listed
    } # end for each $phase
  } # end while 1
} # end phaselist



# kepler - solve the equation of Kepler

sub kepler {
    my ($m, $ecc) = @_;
    my ($e, $delta);
    my $EPSILON = 1e-6;

    $m = torad($m);
    $e = $m;
    do {
        $delta = $e - $ecc * sin($e) - $m;
        $e -= $delta / (1 - $ecc * cos($e));
    } while (abs($delta) > $EPSILON);
    return ($e);
}



# phase - calculate phase of moon as a fraction:
#
# The argument is the time for which the phase is requested,
# expressed as a Julian date and fraction.  Returns the terminator
# phase angle as a percentage of a full circle (i.e., 0 to 1),
# and stores into pointer arguments the illuminated fraction of
# the Moon's disc, the Moon's age in days and fraction, the
# distance of the Moon from the centre of the Earth, and the
# angular diameter subtended by the Moon as seen by an observer
# at the centre of the Earth.

sub phase {
    my $pdate = jtime(shift || time());

    my $pphase;				# illuminated fraction
    my $mage;				# age of moon in days
    my $dist;				# distance in kilometres
    my $angdia;				# angular diameter in degrees
    my $sudist;				# distance to Sun
    my $suangdia;				# sun's angular diameter

    my ($Day, $N, $M, $Ec, $Lambdasun, $ml, $MM, $MN, $Ev, $Ae, $A3, $MmP,
       $mEc, $A4, $lP, $V, $lPP, $NP, $y, $x, $Lambdamoon, $BetaM,
       $MoonAge, $MoonPhase,
       $MoonDist, $MoonDFrac, $MoonAng, $MoonPar,
       $F, $SunDist, $SunAng,
       $mpfrac);

    # Calculation of the Sun's position.

    $Day = $pdate - $Epoch;						# date within epoch
    $N = fixangle((360 / 365.2422) * $Day);				# mean anomaly of the Sun
    $M = fixangle($N + $Elonge - $Elongp);				# convert from perigee
                                    # co-ordinates to epoch 1980.0
    $Ec = kepler($M, $Eccent);					# solve equation of Kepler
    $Ec = sqrt((1 + $Eccent) / (1 - $Eccent)) * tan($Ec / 2);
    $Ec = 2 * todeg(atan($Ec));					# true anomaly
    $Lambdasun = fixangle($Ec + $Elongp);				# Sun's geocentric ecliptic
                                    # longitude
    # Orbital distance factor.
    $F = ((1 + $Eccent * cos(torad($Ec))) / (1 - $Eccent * $Eccent));
    $SunDist = $Sunsmax / $F;					# distance to Sun in km
    $SunAng = $F * $Sunangsiz;					# Sun's angular size in degrees


    # Calculation of the Moon's position.

    # Moon's mean longitude.
    $ml = fixangle(13.1763966 * $Day + $Mmlong);

    # Moon's mean anomaly.
    $MM = fixangle($ml - 0.1114041 * $Day - $Mmlongp);

    # Moon's ascending node mean longitude.
    $MN = fixangle($Mlnode - 0.0529539 * $Day);

    # Evection.
    $Ev = 1.2739 * sin(torad(2 * ($ml - $Lambdasun) - $MM));

    # Annual equation.
    $Ae = 0.1858 * sin(torad($M));

    # Correction term.
    $A3 = 0.37 * sin(torad($M));

    # Corrected anomaly.
    $MmP = $MM + $Ev - $Ae - $A3;

    # Correction for the equation of the centre.
    $mEc = 6.2886 * sin(torad($MmP));

    # Another correction term.
    $A4 = 0.214 * sin(torad(2 * $MmP));

    # Corrected longitude.
    $lP = $ml + $Ev + $mEc - $Ae + $A4;

    # Variation.
    $V = 0.6583 * sin(torad(2 * ($lP - $Lambdasun)));

    # True longitude.
    $lPP = $lP + $V;

    # Corrected longitude of the node.
    $NP = $MN - 0.16 * sin(torad($M));

    # Y inclination coordinate.
    $y = sin(torad($lPP - $NP)) * cos(torad($Minc));

    # X inclination coordinate.
    $x = cos(torad($lPP - $NP));

    # Ecliptic longitude.
    $Lambdamoon = todeg(atan2($y, $x));
    $Lambdamoon += $NP;

    # Ecliptic latitude.
    $BetaM = todeg(asin(sin(torad($lPP - $NP)) * sin(torad($Minc))));

    # Calculation of the phase of the Moon.

    # Age of the Moon in degrees.
    $MoonAge = $lPP - $Lambdasun;

    # Phase of the Moon.
    $MoonPhase = (1 - cos(torad($MoonAge))) / 2;

    # Calculate distance of moon from the centre of the Earth.

    $MoonDist = ($Msmax * (1 - $Mecc * $Mecc)) /
        (1 + $Mecc * cos(torad($MmP + $mEc)));

    # Calculate Moon's angular diameter.

    $MoonDFrac = $MoonDist / $Msmax;
    $MoonAng = $Mangsiz / $MoonDFrac;

    # Calculate Moon's parallax.

    $MoonPar = $Mparallax / $MoonDFrac;

    $pphase = $MoonPhase;
    $mage = $Synmonth * (fixangle($MoonAge) / 360.0);
    $dist = $MoonDist;
    $angdia = $MoonAng;
    $sudist = $SunDist;
    $suangdia = $SunAng;
    $mpfrac = fixangle($MoonAge) / 360.0;
    return wantarray ? ( $mpfrac, $pphase, $mage, $dist, $angdia, $sudist,$suangdia ) : $mpfrac;
}

1;
__END__

=head1 NAME

Astro::MoonPhase - Information about the phase of the Moon

=head1 SYNOPSIS

use Astro::MoonPhase;

    ( $MoonPhase,
      $MoonIllum,
      $MoonAge,
      $MoonDist,
      $MoonAng,
      $SunDist,
      $SunAng ) = phase($seconds_since_1970);

    @phases  = phasehunt($seconds_since_1970);

    ($phase, @times) = phaselist($start, $stop);

=head1 DESCRIPTION

MoonPhase calculates information about the phase of the moon
at a given time.

=head1 FUNCTIONS

=head2 phase()

    ( $MoonPhase,
      $MoonIllum,
      $MoonAge,
      $MoonDist,
      $MoonAng,
      $SunDist,
      $SunAng )  = phase($seconds_since_1970);

      $MoonPhase = phase($seconds_since_1970);

The argument is the time for which the phase is requested,
expressed as a time returned by the C<time> function. If C<$seconds_since_1970>
is omitted, it does C<phase(time)>.

Return value in scalar context is $MoonPhase,
the terminator phase angle as a percentage of a full circle (i.e., 0 to 1).

=over 4

=item B<Return values in array context:>

=item $MoonPhase:

the terminator phase angle as a percentage of a full circle (i.e., 0 to 1)

=item $MoonIllum:

the illuminated fraction of the Moon's disc

=item $MoonAge:

the Moon's age in days and fraction

=item $MoonDist:

the distance of the Moon from the centre of the Earth

=item $MoonAng:

the angular diameter subtended by the Moon as seen by
an observer at the centre of the Earth.

=item $SunDist:

the distance from the Sun in km

=item $SunAng:

the angular size of Sun in degrees

=back

Example:

   ( $MoonPhase,
     $MoonIllum,
     $MoonAge,
     $MoonDist,
     $MoonAng,
     $SunDist,
     $SunAng ) = phase();

     print "MoonPhase  = $MoonPhase\n";
     print "MoonIllum  = $MoonIllum\n";
     print "MoonAge    = $MoonAge\n";
     print "MoonDist   = $MoonDist\n";
     print "MoonAng    = $MoonAng\n";
     print "SunDist    = $SunDist\n";
     print "SunAng     = $SunAng\n";

could print something like this:

     MoonPhase  = 0.598939375319023
     MoonIllum  = 0.906458030827876
     MoonAge    = 17.6870323368022
     MoonDist   = 372479.357420033
     MoonAng    = 0.534682403555093
     SunDist    = 152078368.820205
     SunAng     = 0.524434538105092

=head2 phasehunt()

     @phases = phasehunt($seconds_since_1970);

Finds time of phases of the moon which surround the given
date.  Five phases are found, starting and ending with the
new moons which bound the current lunation.

The argument is the time, expressed as a time returned
by the C<time> function. If C<$seconds_since_1970>
is omitted, it does C<phasehunt(time)>.

Example:

    @phases = phasehunt();
    print "New Moon      = ", scalar(localtime($phases[0])), "\n";
    print "First quarter = ", scalar(localtime($phases[1])), "\n";
    print "Full moon     = ", scalar(localtime($phases[2])), "\n";
    print "Last quarter  = ", scalar(localtime($phases[3])), "\n";
    print "New Moon      = ", scalar(localtime($phases[4])), "\n";

could print something like this:

    New Moon      = Wed Jun 24 06:51:47 1998
    First quarter = Wed Jul  1 21:42:19 1998
    Full moon     = Thu Jul  9 19:02:47 1998
    Last quarter  = Thu Jul 16 18:15:18 1998
    New Moon      = Thu Jul 23 16:45:01 1998

=head2 phaselist()

    ($phase, @times) = phaselist($start, $stop);

Finds times of all phases of the moon which occur on or after
C<$start> but before C<$stop>.  Both the arguments and the return
values are expressed as seconds since 1970 (like the C<time> function
returns).

C<$phase> is an integer indicating the phase of the moon at
C<$times[0]>, as shown in this table:

    0  New Moon
    1  First quarter
    2  Full Moon
    3  Last quarter

The remaining values in C<@times> indicate subsequent phases of the
moon (in ascending order by time).  If there are no phases of the moon
between C<$start> and C<$stop>, C<phaselist> returns the empty list.

Example:

    @name = ("New Moon", "First quarter", "Full moon", "Last quarter");
    ($phase, @times) = phaselist($start, $stop);

    while (@times) {
      printf "%-14s= %s\n", $name[$phase], scalar localtime shift @times;
      $phase = ($phase + 1) % 4;
    }

could produce the same output as the C<phasehunt> example above (given
the appropriate start & stop times).

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
