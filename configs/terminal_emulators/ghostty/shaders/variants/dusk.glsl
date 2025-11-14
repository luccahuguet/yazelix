// Dusk variant: midnight blue core with coral highlights

const vec4 DUSK_BLUE = vec4(0.118, 0.118, 0.184, 1.0); // #1E1E2F
const vec4 DUSK_CORAL = vec4(0.914, 0.271, 0.376, 1.0); // #E94560
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

    vec2 axis = normalize(centerCC - centerCP + 1e-6);
    float u = dot(vu - centerCP, axis);
    float t = clamp(u / max(lineLength, 1e-4), 0.0, 1.0);
    float pulse = 0.05 * sin(iTime * 1.3);

    vec4 blend = mix(DUSK_BLUE, DUSK_CORAL, smoothstep(0.0, 1.0, t));
    vec4 duskBase = mix(DUSK_BLUE, blend, 0.40 + pulse);
    vec4 duskEdge = mix(blend, DUSK_CORAL, 0.32 + pulse * 0.6);

    vec4 trail = fragColor;
    trail = mix(saturate(duskBase, 1.35), trail, 1. - smoothstep(0.0, sdfTrail + mod + 0.010, 0.035));
    trail = mix(saturate(duskEdge, 1.45), trail, 1. - smoothstep(0., sdfTrail + mod, 0.006));
    trail = mix(trail, saturate(duskBase, 1.4), step(sdfTrail + mod, 0.));

    trail = mix(saturate(duskEdge, 1.5), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    trail = mix(saturate(duskBase, 1.45), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}
