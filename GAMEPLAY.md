# Gameplay

This document describes the high-level gameplay and intended user experience of GemGame. For a description of the software from a technical standpoint, please see the `ARCHITECTURE.md` document.

## Overview

* As described in the `README.md` document, GemGame gameplay is centred around the collection of precious gemstones found around a procedurally-generated grid-based game world.
* Gems can be exchanged for items that allow the player to collect gems at a faster rate or slow the rate at which other players can collect gems.

## Gems

* Gems can be found both on the ground (collected simply by walking over them) or embedded in cave walls (collected by bumping into said walls).
* Valuable gems are more likely to be found embedded in walls instead of on the ground.
* Types of gem (from least to most valuable):
  * Emerald
  * Ruby
  * Diamond
* For determining a player's final score, each gem type is worth a tenth of the gem type that follows it when ordered by variety (e.g. 1 diamond is worth 10 rubies or 100 emeralds).
* Gem types cannot be exchanged during gameplay so players will need to collect a variety of gem types if they wish to have access to the various different items available to purchase as each item can only be bought using a certain gem type.

## Items

* Items can be bought at any point during the game.
* Traps appear as gems to other players and are single-use (i.e. can only be triggered once).
* Items:
  * Energy Drink (10 emeralds) - Increases movement speed by 50% (ignoring the effect of running shoes if any) for 10 seconds. This effect does not stack.
  * Running Shoes (25 emeralds) - Increases movement speed by 25%.
  * Bomb (5 rubies) - Can be placed and later detonated. Detonation directly next to a player will cause them to die (when a player dies they will loose some amount of gems before respawning) while any player within the larger blast radius will become temporarily unable to move. Multiple bombs can be placed and then detonated simultaneously.
  * Speed Trap (2 diamonds) - Halves the movement speed of any player who steps on this trap.
  * Theft Trap (5 diamonds) - Takes 25% of the emeralds held by any player that steps on this trap and gives them to the player who set the trap.

## Player Character Appearance

* When a player first joins, their character/entity is given a random appearance.
* A player can change their character's hair style/colour, and skin colour. These changes will be remembered between games (as long as browser local storage is not cleared).
