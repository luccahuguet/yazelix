// Eclipse variant: deep indigo core with golden highlight

const vec4 ECLIPSE_INDIGO = vec4(0.180, 0.161, 0.306, 1.0); // #2E294E
const vec4 ECLIPSE_GOLD = vec4(1.000, 0.831, 0.000, 1.0);   // #FFD400
const float DURATION = 0.22;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderAxisGradientTrail(fragColor, fragCoord, ECLIPSE_INDIGO, ECLIPSE_GOLD, DURATION, 1.6, 0.42, 0.30, 0.5);
}
