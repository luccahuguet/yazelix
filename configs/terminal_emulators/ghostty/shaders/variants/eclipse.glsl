// Eclipse variant: vertical split (deep indigo / golden highlight)

const vec4 ECLIPSE_INDIGO = vec4(0.180, 0.161, 0.306, 1.0); // #2E294E
const vec4 ECLIPSE_GOLD = vec4(1.000, 0.831, 0.000, 1.0);   // #FFD400
const float DURATION = 0.22;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSplitColorTrail(fragColor, fragCoord, ECLIPSE_INDIGO, ECLIPSE_GOLD, DURATION, 0.0, 1.0);
}
