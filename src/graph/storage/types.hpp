#include <cstddef>
#include <cstdint>

typedef std::uint32_t NodeID;
typedef std::uint32_t EdgeID;
typedef std::int32_t Weight;

// CH graph

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

// RTree

typedef std::int32_t FixedLongitude;
typedef std::int32_t FixedLatitude;

struct RectangleInt2D
{
    FixedLongitude min_lon, max_lon;
    FixedLatitude min_lat, max_lat;
};

struct TreeNode
{
    RectangleInt2D minimum_bounding_rectangle;
};

typedef std::uint64_t TreeLevelStart;

struct Coordinate
{
    FixedLongitude longitude;
    FixedLatitude latitude;
};

// fileIndex

struct SegmentID
{
    NodeID id : 31;
    bool enabled : 1;
};

struct EdgeBasedNodeSegment
{
    SegmentID forward_segment_id;        // edge-based graph node ID in forward direction (u->v)
    SegmentID reverse_segment_id;        // edge-based graph node ID in reverse direction (v->u if exists)
    NodeID u;                            // node-based graph node ID of the start node
    NodeID v;                            // node-based graph node ID of the target node
    unsigned short fwd_segment_position; // segment id in a compressed geometry
};
