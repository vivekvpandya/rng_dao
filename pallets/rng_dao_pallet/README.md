License: Unlicense

Random Number Generation Pallet:

Flow:

- 1. A user who wants to generate random number, will create a rng_cycle with at least minimum bounty
     specified in runtime Configuration.
- 2. After this process is divide in two phases:

  - a) Any user who wants to paricipate in the random number generation process can submit a
    Keccke256 has of a number with a deposit value as specified in runtime configuration.
    Here in first phase after certain delay a bot user can also participate.
  - b) There is dealy between second phase starts, thus it gives sometime to anyone participate.
    Once the dealy is over second phase starts where, participants from earlier phase
    reveal there secret number. If they fail to do so in certain time duration then their
    deposit is slahsed. If participants reveal correct secret number they get their deposit
    back and share from bounty.

- 3. A after the deadline of the cycle if at least one participant has revealed correct secret number
     system have a random number generated for given cycle.
- 4. If there are no participants or none of them revealed correctly then cycle fails and creator gets her bounty back.

Situation described in 4) can be aovided by having a good value for a deposit so that participants
are required to reveal correct secret number otherwise they loose.
Also additional bot users will avoid condition where there are no participants.

In current implementaion bots are not treated differently but in future it can be changed, such as no bounty share provided to bots.

So in this system a cycle will complete in eaxct number of blocks configured in various deadlines runtime Config.
A user is incentived to take part as they get share from bounty value.
