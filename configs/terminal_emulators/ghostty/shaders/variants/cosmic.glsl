// Cosmic purple variant

const vec4 TRAIL_COLOR = vec4(0.78, 0.38, 0.96, 1.0);      // ~#C764F5
const vec4 TRAIL_COLOR_ACCENT = vec4(0.47, 0.24, 0.86, 1.0); // ~#783DDC
const float DURATION = 0.28;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSimpleDualColorTrail(fragColor, fragCoord, TRAIL_COLOR, TRAIL_COLOR_ACCENT, DURATION, .006, 1.4);
}
