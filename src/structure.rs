
struct Entry {
    
}




struct Dam {
    db: rusqlite::Connection,
    import: std::path::PathBuf,
    remotes: Vec<url::Url>
}

impl Dam {
    fn init(path) -> Result<Self>;

    fn open(path) -> Result<Self>;

    fn clone(url, path) -> Result<Self>;

    fn import() -> Result<(), ()>;

    fn list() -> Result<Vec<Entry>, ()>;

    fn find(query) -> Result<Vec<Entry>, ()>;

    fn open(query) -> Result<(), ()>;

    fn add(path) -> Result<(), ()>;

    fn remove(entry) -> Result<(), ()>;

    fn info(entry) -> Result<(), ()>;

    fn start() -> Result<(), ()>;
}