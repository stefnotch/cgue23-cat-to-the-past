#define GROUP_SIZE         8
#define GROUP_THREAD_COUNT (GROUP_SIZE * GROUP_SIZE)
#define FILTER_RADIUS      (FILTER_SIZE / 2)
#define TILE_SIZE          (GROUP_SIZE + 2 * FILTER_RADIUS)
#define TILE_PIXEL_COUNT   (TILE_SIZE * TILE_SIZE)

shared float shared_memory_r[TILE_PIXEL_COUNT];
shared float shared_memory_g[TILE_PIXEL_COUNT];
shared float shared_memory_b[TILE_PIXEL_COUNT];

void store_sample(int index, vec3 color) {
    shared_memory_r[index] = color.r;
    shared_memory_g[index] = color.g;
    shared_memory_b[index] = color.b;
}

vec3 load_sample(uint center_index, ivec2 offset) {
    uint index = center_index + offset.y * TILE_SIZE + offset.x;
    return vec3(
        shared_memory_r[index],
        shared_memory_g[index],
        shared_memory_b[index]
    );
}

// | uvec3 | gl_NumWorkGroups        | number of work groups that have been dispatched set by glDispatchCompute()                                                                                                                  |
// |-------|-------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
// | uvec3 | gl_WorkGroupSize        | size of the work group (local size) operated on defined with layout                                                                                                                         |
// | uvec3 | gl_WorkGroupID          | index of the work group currently being operated on                                                                                                                                         |
// | uvec3 | gl_LocalInvocationID    | index of the current work item in the work group                                                                                                                                            |
// | uvec3 | gl_GlobalInvocationID   | global index of the current work item  (gl_WorkGroupID * gl_WorkGroupSize + gl_LocalInvocationID)                                                                                           |
// | uint  | gl_LocalInvocationIndex | 1d index representation of gl_LocalInvocationID  (gl_LocalInvocationID.z * gl_WorkGroupSize.x * gl_WorkGroupSize.y  + gl_LocalInvocationID.y * gl_WorkGroupSize.x + gl_LocalInvocationID.x) |