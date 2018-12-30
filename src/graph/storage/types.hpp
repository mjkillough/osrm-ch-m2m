#include <cstddef>
#include <cstdint>

struct EdgeArrayEntry
{
    std::uint32_t target;

    std::uint32_t turn_id : 31;
    bool shortcut : 1;
    std::int32_t weight;
    std::int32_t duration : 30;
    bool forward : 1;
    bool backward : 1;
};

struct NodeArrayEntry
{
    std::uint32_t first_edge;
};

struct Metadata
{
    std::uint64_t element_count;
};

const auto EDGE_ARRAY_ENTRY_SIZE = sizeof(EdgeArrayEntry);
const auto NODE_ARRAY_ENTRY_SIZE = sizeof(NodeArrayEntry);
const auto METADATA_SIZE = sizeof(Metadata);
