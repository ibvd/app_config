### Error Handling

Redo Error handling via something like error_chain or anyhow or eyre
- Or even better figure out how to easily bubble up custom errors w/o
- a lot of cognative effort. A little boilerplate is ok.

### AWS Paramstore as a provider

### Params in template files

Put a helper in the template processor to allow a template to directly look up keys in SSM ParamStore.  That way not 100% of information must come from the Provider.  This makes the template much more like consul templates.

### Async 

Tokio is a lot of overhead for us to just hit a cloud API.  See if we can
can switch to something like [smol](https://github.com/smol-rs/smol/blob/master/examples/async-h1-client.rs)