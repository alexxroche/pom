# pom
Phases Of Moon - calculate when the next Full Moon will be

I have been using https://metacpan.org/pod/Astro::MoonPhase
since 2007 in a small perl wrapper called pom.

I thought that I should port it to Rust and this is my
first attempt. It was surprisingly easy. I expect that it
might be possible to automate porting many perl modules
to Rust.

# performance
```txt
perl> time pom # ver. 0.2
real    0m0.022s
user    0m0.022s
sys     0m0.000s

rust> time pom # ver. 0.3
real    0m0.002s
user    0m0.002s
sys     0m0.001s
```
