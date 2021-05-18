Rust Console Game Engine
========================
This is a **hobby** project. It's a console game engine for Rust. Heavily inspired by [javidx9's One Lone Coder Console Game Engine](https://github.com/OneLoneCoder/videos/blob/master/olcConsoleGameEngine.h). The goal is to learn Rust and a bit about making games.

For now, it only works on Windows.

Requirements
------------
* windows
* rustc

Usage
-----
`cargo build` one of the examples from **examples/\*\***. The actual engine is in **src/lib.rs**.

Debugging with VSCode & rust-analyser
-------------------------------------
The following settings need to be in your ```settings.json``` file :

    {
        "rust-analyzer.linkedProjects": [
            "Cargo.toml",
            "examples/fps/Cargo.toml",
            "examples/noise/Cargo.toml",
            "examples/racer/Cargo.toml",
            "examples/test_engine/Cargo.toml"
        ]
    }

The following tasks need to be in your ```tasks.json``` file : 

    {
        "type": "cargo",
        "label": "fps",
        "command": "build",
        "problemMatcher": [
            "$rustc"
        ],
        "group": "build",
        "options": {
            "cwd": "${workspaceFolder}/examples/fps"
        }
    },
    {
        "type": "cargo",
        "label": "noise",
        "command": "build",
        "problemMatcher": [
            "$rustc"
        ],
        "group": "build",
        "options": {
            "cwd": "${workspaceFolder}/examples/noise"
        }
    },
    {
        "type": "cargo",
        "label": "racer",
        "command": "build",
        "problemMatcher": [
            "$rustc"
        ],
        "group": "build",
        "options": {
            "cwd": "${workspaceFolder}/examples/racer"
        }
    }

The following configuration needs to be in your ```launch.json``` file :

    {
        "name": "fps",
        "type": "cppvsdbg",
        "request": "launch",
        "program": "${workspaceFolder}/examples/fps/target/debug/fps.exe",
        "args": [],
        "stopAtEntry": false,
        "cwd": "${workspaceFolder}/examples/fps",
        "environment": [],
        "console": "newExternalWindow",
        "preLaunchTask": "fps"
    },
    {
        "name": "noise",
        "type": "cppvsdbg",
        "request": "launch",
        "program": "${workspaceFolder}/examples/noise/target/debug/noise.exe",
        "args": [],
        "stopAtEntry": false,
        "cwd": "${workspaceFolder}/examples/noise",
        "environment": [],
        "console": "newExternalWindow",
        "preLaunchTask": "noise"
    },
    {
        "name": "racer",
        "type": "cppvsdbg",
        "request": "launch",
        "program": "${workspaceFolder}/examples/racer/target/debug/racer.exe",
        "args": [],
        "stopAtEntry": false,
        "cwd": "${workspaceFolder}/examples/racer",
        "environment": [],
        "console": "newExternalWindow",
        "preLaunchTask": "racer"
    }
