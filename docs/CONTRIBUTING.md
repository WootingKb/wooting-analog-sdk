### Contributing Note

The headers generated for the SDK crate are verified in the CI to ensure that the current headers
are up to date and that we can review any changes which are made to the headers (rather than purely
generating them and potentially not knowing exactly what has changed). Before commiting (if you've
made changes to any part of the SDK crate) you should run 
`cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h --verify` to ensure
that your headers are up to date, if this fails due to them being different, run 
`cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h` and review the
changes to the headers before commiting.