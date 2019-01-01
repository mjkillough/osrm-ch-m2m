#include <cstddef>
#include <cstdint>

// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/util/typedefs.hpp#L72-L81
using NodeID = std::uint32_t;
using EdgeID = std::uint32_t;
using Weight = std::int32_t;

// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/util/static_graph.hpp#L55
// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/contractor/query_edge.hpp#L17
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

// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/util/static_graph.hpp#L47
struct NodeArrayEntry
{
    EdgeID first_edge;
};

// https://github.com/Project-OSRM/osrm-backend/blob/5.18/include/storage/io.hpp#L162-L167
struct Metadata
{
    std::uint64_t element_count;
};
