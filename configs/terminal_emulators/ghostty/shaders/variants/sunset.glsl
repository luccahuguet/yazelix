// Sunset orange/pink variant

const vec4 TRAIL_COLOR = vec4(1.00, 0.48, 0.35, 1.0);      // ~#FF7A59
const vec4 TRAIL_COLOR_ACCENT = vec4(1.00, 0.24, 0.37, 1.0); // ~#FF3D5E
const float DURATION = 0.27;

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

    float mod = .006;
    vec4 trail = mix(saturate(TRAIL_COLOR_ACCENT, 1.5), fragColor, 1. - smoothstep(0., sdfTrail + mod, 0.007));
    trail = mix(saturate(TRAIL_COLOR, 1.5), trail, 1. - smoothstep(0., sdfTrail + mod, 0.006));
    trail = mix(trail, saturate(TRAIL_COLOR, 1.4), step(sdfTrail + mod, 0.));
    trail = mix(saturate(TRAIL_COLOR_ACCENT, 1.5), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    trail = mix(saturate(TRAIL_COLOR, 1.5), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}
