# unfolded-tabletop

Concrete mechanisms shared by named consumer games, and the terms used for how a game takes and keeps them.

## Language

**Consumer**:
A repository whose build depends on a package from this one. Every current consumer is a game.
_Avoid_: client, user, downstream (say "game" only where the thing really is a game)

**Mechanism**:
One concrete, game-neutral capability owned here, such as seeded randomness or the agent seat.
_Avoid_: feature, component, framework

**Adapter**:
The code in a consumer that connects a mechanism to that consumer's own policy: what it observes and names, its layout, its admission, its wording.
_Avoid_: integration, wrapper, glue

**Pin**:
The one immutable Git `rev` at which a consumer depends on every package it takes from here.
_Avoid_: version, release

**Agent seat**:
An agent sitting at a game's real table with no window, through one JSON request and one JSON response per line. Its table still has a logical primary window with nothing behind it (`capture::install`), so code that reads the primary window runs at the seat too.
_Avoid_: bot, harness, driver

**Agent player**:
An external agent that plays at a table someone else is running, by sending requests of the game's own protocol to a socket the running game opens. Each game has its own word for one, such as Arkham's agent partner.
_Avoid_: agent seat (the windowless table an agent sits at alone), bot, client

**Held start**:
A start of the agent seat whose session is kept open on a local socket across shell commands, rather than answered on standard input and output.
_Avoid_: daemon, server mode
