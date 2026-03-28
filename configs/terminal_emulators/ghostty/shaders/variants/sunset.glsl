// Sunset orange/pink variant

const vec4 TRAIL_COLOR = vec4(1.00, 0.48, 0.35, 1.0);      // ~#FF7A59
const vec4 TRAIL_COLOR_ACCENT = vec4(1.00, 0.24, 0.37, 1.0); // ~#FF3D5E
const float DURATION = 0.27;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSimpleDualColorTrail(fragColor, fragCoord, TRAIL_COLOR, TRAIL_COLOR_ACCENT, DURATION, .006, 1.4);
}
