# osrm-ch-m2m

A re-implementation of [`osrm-backend`](https://github.com/Project-OSRM/osrm-backend)'s many-to-many ('table') query for Contraction Hierarchy graphs.

The OSM data must be pre-processed into a Contraction Hierarchy (CH) graph by the `osrm-backend` tools as normal. The same file-format is used by this code, which means it is tightly coupled to the version of OSRM that it is based off: v5.18.

This re-implementation does not take care of finding the nearest-neighbour graph nodes to the input co-ordinates. The many-to-many query must first be run through a [modified version of `osrm-backend`](https://github.com/Project-OSRM/osrm-backend/compare/5.18...mjkillough:hacks) to get the `PhantomNode` entries for each coordinate (called a `heap::Query` in the Rust code). These are loaded in through `queries.json`.

## Why?

The `osrm-backend` code is very high-quality, optimised, modern C++. There's no significant benefit to re-writing it in Rust - in fact, this re-implementation is slower and probably riddled with bugs!

This was done as an attempt to learn how the Contraction Hierarchies many-to-many query works. There was hope that in doing so, some opportunities for optimisation would become apparent (see [Parallelism](#parallelism), below).

## Parallelism

The many-to-many query of Sources to Targets is performed as bidrectional Dijkstra searches. There are `|Target|` backwards searches, and `|Source|` forward searches. The backwards searches must be complete before the forward searches can be done, but each of the forward/backward searches are otherwise independent.

This implementation parallelises each of the forward and backward searches. For large matrices, this brings a significant speed-up. (I believe older versions of `osrm-backend` also had code to do this, but it was [removed](https://github.com/Project-OSRM/osrm-backend/issues/4560)).

## Is it safe Rust?

Although the algorithm itself is written in safe Rust, the file format for the OSRM graph is based on the in-memory representation of `osrm-backend`'s C++ objects, which involve optimisations like bit-packing.

As the layout of C++ objects is not well defined, this uses `bindgen` to generate bindings that account for the platform's layout strategies. This code must be compiled on the same platform (and ideally with the same compiler?) that compiled the `osrm-backend` that generated the CH graph.

## Should I use this?

No.

