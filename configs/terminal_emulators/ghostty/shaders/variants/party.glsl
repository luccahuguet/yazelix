// Party variant: vivid multi-hue neon with animated HSV palette

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

const float DURATION = 0.24;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    #if !defined(WEB)
    fragColor = texture(iChannel0, fragCoord.xy / iResolution.xy);
    #endif
    vec2 vu = normalize(fragCoord, 1.);
    vec2 offsetFactor = vec2(-.5, 0.5);

    vec4 currentCursor = vec4(normalize(iCurrentCursor.xy, 1.), normalize(iCurrentCursor.zw, 0.));
    vec4 previousCursor = vec4(normalize(iPreviousCursor.xy, 1.), normalize(iPreviousCursor.zw, 0.));

    vec2 centerCC = getRectangleCenter(currentCursor);
    vec2 centerCP = getRectangleCenter(previousCursor);
    float vertexFactor = determineStartVertexFactor(currentCursor.xy, previousCursor.xy);
    float invertedVertexFactor = 1.0 - vertexFactor;

    vec2 v0 = vec2(currentCursor.x + currentCursor.z * vertexFactor, currentCursor.y - currentCursor.w);
    vec2 v1 = vec2(currentCursor.x + currentCursor.z * invertedVertexFactor, currentCursor.y);
    vec2 v2 = vec2(previousCursor.x + currentCursor.z * invertedVertexFactor, previousCursor.y);
    vec2 v3 = vec2(previousCursor.x + currentCursor.z * vertexFactor, previousCursor.y - previousCursor.w);

    float sdfCurrentCursor = getSdfRectangle(vu, currentCursor.xy - (currentCursor.zw * offsetFactor), currentCursor.zw * 0.5);
    float sdfTrail = getSdfParallelogram(vu, v0, v1, v2, v3);

    float progress = clamp((iTime - iTimeCursorChange) / DURATION, 0.0, 1.0);
    float easedProgress = ease(progress);
    float lineLength = distance(centerCC, centerCP);

    float mod = .005;

    // Parameter along the trail axis for color blend
    vec2 axis = normalize(centerCC - centerCP + 1e-6);
    float u = dot(vu - centerCP, axis);
    float t = clamp(u / max(lineLength, 1e-4), 0.0, 1.0);
    // Animate slightly for a living neon feel
    float hue = fract(t * 0.85 + iTime * 0.12);
    vec4 neonBase = vec4(hsv2rgb(vec3(hue, 1.0, 1.0)), 1.0);
    vec4 neonEdge = vec4(hsv2rgb(vec3(fract(hue + 0.12), 1.0, 1.0)), 1.0);

    // Build vibrant core, bright edges, and soft outer glow
    vec4 trail = fragColor;
    // Outer glow
    trail = mix(saturate(neonBase, 1.6), trail, 1. - smoothstep(0.0, sdfTrail + mod + 0.010, 0.035));
    // Edge highlight
    trail = mix(saturate(neonEdge, 1.7), trail, 1. - smoothstep(0., sdfTrail + mod, 0.006));
    // Core fill
    trail = mix(saturate(neonBase, 1.6), trail, step(sdfTrail + mod, 0.));

    // Cursor core and edge pop
    trail = mix(saturate(neonEdge, 1.8), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    trail = mix(saturate(neonBase, 1.7), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}
