#include <cstddef>
#include <cstdint>

using NodeID = std::uint32_t;
using EdgeID = std::uint32_t;
using Weight = std::int32_t;

struct EdgeArrayEntry
{
    NodeID target;

    NodeID turn_id : 31;
    bool shortcut : 1;
    Weight weight;
    Weight duration : 30;
    bool forward : 1;
    bool backward : 1;
};

struct NodeArrayEntry
{
    EdgeID first_edge;
};

struct Metadata
{
    std::uint64_t element_count;
};
