I believed I had found a strategy for capturing a [Korath World-Ship](http://endless-spoiler/ship/korath_world_ship.html) in [Endless Sky](http://endless-sky.github.io/). I was wrong. This is the simulator I used to prove it.

Build it like any other Rust program, using `cargo build --release`. The usage string hopefully does a good job of explaining how to use it:

```
Usage: capsim options...

Options:
    -ucount COUNT       Specify the number of work units to run. Default is
                        '100'.
    -icount COUNT       Specify the number of iterations per work unit. Larger
                        values are less granular but more efficient. Default
                        is '100'.
    -tcount COUNT       Specify the number of threads to use. Default is 2,
                        which is how many CPUs this computer seems to have.
    -ucrew CREW         Specify the starting crew quantity on the player's
                        ship. Required.
    -mcrew CREW         Specify the starting crew quantity on the enemy's
                        ship. Required.
    -ugov ATTACK/DEFENSE
                        Specify the player's government's intrinsic attack and
                        defense strengths. The default is '1.0/2.0', the only
                        value this will ever have in vanilla.
    -mgov ATTACK/DEFENSE
                        Specify the enemy government's intrinsic attack and
                        defense strengths. The default is '1.0/2.0', the most
                        common values. Alpha and Korath governments have
                        higher values.
    -uwep WEAPON x COUNT
                        Specify a type of weapon on the player's ship, e.g.
                        'Laser Rifle x 47'. This option may be specified more
                        than once.
    -mwep WEAPON x COUNT
                        Specify a type of weapon on the enemy's ship, e.g.
                        'Korath Repeater Rifle x 150'. This option may be
                        specified more than once.


Example invocation, in which a souped-up Bactrian is attacking a harrowed
World-Ship:

capsim \
    -ucrew 461 -uwep "Pug Biodefenses x 150" -uwep "Nerve Gas x 461" \
    -uwep "Tuning Rifle x 245" \
    -mcrew 799 -mgov 1.4/2.6 -mwep "Korath Repeater Rifle x 150"

Here are the weapons I know about:
	Fragmentation Grenades (attack 1.3, defense 0.3)
	Intrusion Countermeasures (attack 0, defense 60)
	Korath Repeater Rifle (attack 1.6, defense 2.4)
	Laser Rifle (attack 0.6, defense 0.8)
	Nerve Gas (attack 2.8, defense 0.8)
	Pug Biodefenses (attack 0, defense 250)
	Pulse Rifle (attack 0.7, defense 1)
	Security Station (attack 0, defense 3.4)
	Tuning Rifle (attack 1.2, defense 1.8)
```

This repository is in the public domain. The code is a hack. Why I went to the trouble of making a whole big multithreaded simulator out of this is anybody's guess.
