# speng-hunter

A pretty easy to use macro/script for SpaceEngine, mainly tailored for people wanting to find some of the rarest finds possible (1.000 ESI Earth-likes, High-ESI Minigiants, >1Ms A|B Flips, etc)

## Workshop notice

If you're here from the workshop, to download this, you can navigate to Steam/steamapps/workshop/content/314650/2898001755 and copy speng-hunter.exe to some other location, and run it. Or, you can go to the releases tab, and download the latest version there.

## Anti-virus notice

Your anti-virus (probably Defender) will, most likely, go off on this. This is a false positive of course. Windows generally has a tendency to flag anything that messes with the mouse/keyboard (AHK, pyautogui, etc) as a virus, which is what this does here. Simply give it an exception in Defender or whichever anti-virus you use.

## How to use

There are quite a few dumb requirements for this program. So, here's a list of them:

* SE (abbreviation of SpaceEngine)'s main window should be as small as possible
* You must be running specifically version 0.990.45.1945.
* If you're on Linux, [run SE and speng-hunter through the same instance of Proton](https://gist.github.com/michaelbutler/f364276f4030c5f449252f2c4d960bd2)

If you've done all that, it should work fine!

Note that prior to running this, you must have filters and such all ready to go! If not, it will search with the wrong filters, and you'll need to restart it once your filters are correct.

This also moves the mouse around, so it can sometimes be quite difficult to close it. I'd recommend quickly tabbing into the terminal it opens and pressing CTRL+C to close it after it starts a search.

Here's all the rare finds speng-hunter will log as "rare":

* 0.998+ ESI
* 1 mass 1 radius
* High-ESI minigiants
* Decent hyperterrestrials
* Decent A|B flips
* 0.990+ ESI earth-likes
* 6km+ deep O2 oceans
* Binary earth-likes (if I can fix this check)

If you have anything I could add here (within reason), let me know! I'll consider it.

Note that speng-hunter requires you to use good filters, so here's some I recommend for the 8 rare things I listed above:

### 0.998 ESI

Any planet with an ESI above 0.998

* System's main star: White dwarfs or G9s (If you want just 1.000 ESIs or want to contribute to finding the 1.000 ESI Earth-like. Please do)
* ESI: 0.990 to 1.000 (This is available with my other project, [speng-starb](https://github.com/Centri3/speng-starb))

If you prefer vanilla SE, here's a good alternative, though which is slower:

* Mass: 0.975 to 1.025
* Radius: 12650 to 12850
* Temp: -30 to 30

This will skip little to no 0.998+ ESIs, though is quite a bit slower (and you run the chance of skipping a 1.000 ESI).

### 1 mass 1 radius

This is pretty simple.

* System's main star: White dwarfs
* Mass: 0.995 to 1.005
* Radius: 12725 to 12775

This should skip 0 1.000s, so if you get lucky, you can also find 1.000s with this!

### High-ESI Minigiants

Minigiants with an ESI above 0.988. Note that these may be impossible, so I wouldn't recommend searching for these

* System's main star: G-types or F-types
* ESI: 0.990 to 1.000 (You can also do Mass, Radius, and Temp like with 0.998 ESI)
* Atm pressure: 1000 to inf

### Decent Hyperterrestrials

The first bug here! These are terrestrial planets with a mass above ~31.8Me (though some "weak" ones can be as low as ~15Me), which are thought to be ice giants which have an atm pressure below 1000.

* System's main star: F-giants
* Mass: 31.8 to inf
* Atm pressure: 0 to 1000

### Decent A|B Flips

These are binary giant planets with a mass above ~13Mj, which is because SE flips the mass/sma of a binary twice or not at all, rather than once, hence the name.

* System's main star: F-types or above
* Mass: 4450 to inf
* Atm pressure: 1000 to inf

### 0.990+ ESI Earth-likes

Pretty self explanatory. Not gonna list good filters here, since they've already been listed in the 0.998 ESIs section! Just make sure to use G9 as System's main star.
