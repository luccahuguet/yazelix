// Neon variant with multi-color blending

const vec4 NEON_CYAN = vec4(0.20, 1.00, 0.00, 1.0);
const vec4 NEON_MAGENTA = vec4(1.00, 0.00, 1.00, 1.0);
const vec4 NEON_LIME = vec4(0.67, 1.00, 0.20, 1.0);
const float DURATION = 0.22;

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

    // Blend classic neon colors along the trail axis with a very subtle time pulse
    vec2 axis = normalize(centerCC - centerCP + 1e-6);
    float u = dot(vu - centerCP, axis);
    float t = clamp(u / max(lineLength, 1e-4), 0.0, 1.0);
    float pulse = 0.08 * sin(iTime * 1.2);

    vec4 c1 = mix(NEON_CYAN, NEON_MAGENTA, smoothstep(0.0, 1.0, t));
    vec4 c2 = mix(NEON_MAGENTA, NEON_LIME, smoothstep(0.0, 1.0, t));
    vec4 neonBase = mix(c1, c2, 0.35 + pulse);
    vec4 neonEdge = mix(neonBase, NEON_MAGENTA, 0.25);

    // Build glow with restrained intensity
    vec4 trail = fragColor;
    trail = mix(saturate(neonBase, 1.4), trail, 1. - smoothstep(0.0, sdfTrail + mod + 0.010, 0.035));
    trail = mix(saturate(neonEdge, 1.5), trail, 1. - smoothstep(0., sdfTrail + mod, 0.006));
    trail = mix(trail, saturate(neonBase, 1.45), step(sdfTrail + mod, 0.));

    // Cursor core and edge
    trail = mix(saturate(neonEdge, 1.55), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    trail = mix(saturate(neonBase, 1.5), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}
