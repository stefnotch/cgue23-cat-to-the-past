// Rewinds time in world space
vec3 timeRewindPosition(vec3 position) {
    // We can't use the normal, because it's not smooth. I wish we had smooth normals for effects like this one.
    // And maybe for the shadow mapping as well?
    // That computation could be done in object space
    // vec3 pos = position + normal * sin(scene.rewindTime * 1.0) * 0.5;

    vec3 center = vec3(0, 0, 0);
    vec3 centerToPosition = position - center;
    float distanceFromCenter = length(centerToPosition);
    centerToPosition = normalize(centerToPosition);

    float scaleFactor = log((distanceFromCenter + 1.0) / 5.0); // Or maybe use a sin function?
    return position + centerToPosition * sin(scene.rewindTime) * scaleFactor;
}