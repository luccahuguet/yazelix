// Ember variant: vertical split (flame orange ↑, deep navy ↓)

const vec4 EMBER_FLAME = vec4(1.0, 0.271, 0.0, 1.0);   // #FF4500
const vec4 EMBER_NAVY = vec4(0.102, 0.102, 0.180, 1.0);  // #1A1A2E
const float DURATION = 0.24;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderVerticalSplitTrail(fragColor, fragCoord, EMBER_FLAME, EMBER_NAVY, DURATION, 1.8);
}
