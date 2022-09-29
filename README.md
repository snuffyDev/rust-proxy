# Rust Proxy Server

This audio proxy server for YouTube/YouTube Music is still an early work-in-progress, use with caution!

> Note: Since this proxy is potentially unstable, a binary is not provided for the time being.
>
> In order to use this you must compile it locally!

The server will listen on port 3001.

## Features

- Works with `/videoplayback` URLs
- Rewrites and proxys HLS streams
- Small binary size (3.65MB on Windows)
- Uses very little resources

## Future Tasks

- Stream the HTTP Responses
- Write tests
- Better error handling
- Remove blacklisted headers
- Use Environment Variables to specify configuration options
