// Orchid variant: hard vertical split (molten amber / cobalt flash)

const vec4 ORCHID_AMBER = vec4(1.0, 0.420, 0.0, 1.0);      // #FF6B00
const vec4 ORCHID_COBALT = vec4(0.126, 0.427, 0.808, 1.0); // #206DCE
const float DURATION = 0.25;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSplitColorTrail(fragColor, fragCoord, ORCHID_AMBER, ORCHID_COBALT, DURATION, 0.0, 0.0);
}
