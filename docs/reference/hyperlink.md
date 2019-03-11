mono_title: true

# hyperlink

    hyperlink(uri: str, size: coord) -> frame

Create a rectangular area of size `size` that opens the given
<span class="smcp">URI</span> when clicked. The frame itself has no visual
artifacts, it can be added over text or over a rectangle, for example. The
origin of the clickable area is in the top-left corner.
