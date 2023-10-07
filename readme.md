# GWYF Helper

A helper for Golf With Your Friends

![screenshot](screenshot.png)

`gwyf-helper`: Injects an embedded copy of `injected-dll` into `Golf With Your Friends.exe` using the usual `LoadLibrary` method

`injected-dll`: Main helper logic. Runs inside GWYF for easy access to unity, game data etc.

## Features

* Scoreboard

* Player + hole ESP

* Automatically copy someone's color

* 'bam'

## Notes

* Run after game has started so it has time to initialise everything

* [Il2CppInspector Tutorial: Working with code in IL2CPP DLL injection projects](https://katyscode.wordpress.com/2021/01/14/il2cppinspector-tutorial-working-with-code-in-il2cpp-dll-injection-projects/)

* ~~Will need to manually change offsets after each GWYF update~~ semi-automated

* Possible todo: move offsets + build stuff to a separate library so injected-dll doesn't take ages to compile
