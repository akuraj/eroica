# eroica

Chess Engine written in Rust. My goal is to write a strong Chess Engine as a learning experience. Yes - it can play decent Chess already, but can't play a whole game yet.

## Current

* Magic Bitboard based move generator
* Heuristic Evaluation using Piece-Square-Tables (PST)
* Alpha-Beta search with Transposition Table
* Ad-Hoc communication protocol implemented for testing
* Move ordering by SEE (Static Exchange Evaluation)
* Basic Quiescence Search

## Next

* Implement a standard chess protocol: CECP or UCI
* Better evaluation function using all of the attack and defend maps computed as part of move generation (eventually implement a Machine Learning algorithm to do the heuristic evaluation)
* Better move-ordering
* Search extensions and other improvments in Quiescence Search
* Late Move Pruning, Late Move Reductions, Null Move Reductions etc.
* Transposition Table cleanup scheme

I try to work on this whenever I get a decent amount of free time (which is not often these days).

Jayakiran
