BEGIN;

CREATE TABLE _SqliteDatabaseProperties (
    key TEXT,
    value TEXT,
    UNIQUE(key)
);

CREATE TABLE ABPersonSearchKey (
    person_id INTEGER,
    SearchKey BLOB,
    NameOnlySearchKey BLOB,
    UNIQUE(person_id)
);

CREATE TABLE ABGroupMembers (
    UID INTEGER PRIMARY KEY,
    group_id INTEGER,
    member_type INTEGER,
    member_id INTEGER,
    UNIQUE(group_id, member_type, member_id)
);

CREATE TABLE ABMultiValue (
    UID INTEGER PRIMARY KEY,
    record_id INTEGER,
    property INTEGER,
    identifier INTEGER,
    label INTEGER,
    value TEXT,
    guid TEXT NOT NULL,
    UNIQUE(guid)
);

CREATE TABLE ABMultiValueLabel (
    value TEXT,
    UNIQUE(value)
);

CREATE TABLE ABMultiValueEntry (
    parent_id INTEGER,
    key INTEGER,
    value TEXT,
    UNIQUE(parent_id, key)
);

CREATE TABLE ABMultiValueEntryKey (
    value TEXT,
    UNIQUE(value)
);

CREATE TABLE ABPhoneLastFour (
    multivalue_id INTEGER PRIMARY KEY,
    value TEXT
);

CREATE TABLE ABPersonMultiValueDeletes (
    record_id INTEGER,
    property_id INTEGER,
    identifier INTEGER,
    sequence_number INTEGER,
    record_uid TEXT,
    identifier_uid TEXT
);

CREATE TABLE ClientCursor (
    ROWID INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id INTEGER,
    latest_consumed_sequence_number INTEGER,
    latest_consumed_timestamp REAL,
    client_identifier TEXT,
    UNIQUE(client_identifier, store_id)
);

CREATE TABLE ClientSequence (
    sequence_number INTEGER,
    client_identifier TEXT,
    save_timestamp REAL
);

CREATE TABLE ClientCursorConsumed (
    store_id INTEGER,
    consumed_entity_class INTEGER,
    consumed_entity_id INTEGER,
    consumed_change_id INTEGER,
    sequence_number INTEGER,
    client_identifier TEXT
);

CREATE TABLE LimitedAccess (
    BundleID TEXT NOT NULL,
    guid TEXT NOT NULL,
    SequenceNumber INTEGER,
    IsActive INTEGER,
    PRIMARY KEY (BundleID, guid)
);

CREATE TABLE ABStore (
    ROWID INTEGER PRIMARY KEY AUTOINCREMENT,
    Name TEXT,
    ExternalIdentifier TEXT,
    Type INTEGER,
    ConstraintsPath TEXT,
    ExternalModificationTag TEXT,
    ExternalSyncTag TEXT,
    StoreInternalIdentifier TEXT,
    AccountID INTEGER DEFAULT -1,
    Enabled INTEGER DEFAULT 1,
    SyncData BLOB,
    MeIdentifier INTEGER DEFAULT -1,
    Capabilities INTEGER DEFAULT 0,
    guid TEXT NOT NULL,
    LastSyncDate TEXT,
    ProviderIdentifier TEXT,
    ProviderMetadata BLOB,
    UNIQUE(StoreInternalIdentifier),
    UNIQUE(guid)
);

CREATE TABLE ABAccount (
    ROWID INTEGER PRIMARY KEY AUTOINCREMENT,
    AccountIdentifier TEXT,
    Flags INTEGER,
    DefaultSourceID INTEGER,
    guid TEXT NOT NULL,
    UNIQUE(guid)
);

CREATE TABLE ABGroup (
    ROWID INTEGER PRIMARY KEY AUTOINCREMENT,
    Name TEXT,
    ExternalIdentifier TEXT,
    StoreID INTEGER,
    ExternalModificationTag TEXT,
    ExternalRepresentation BLOB,
    ExternalUUID TEXT,
    guid TEXT NOT NULL,
    UNIQUE(guid)
);

CREATE TABLE ABGroupChanges (
    record INTEGER,
    type INTEGER,
    sequence_number INTEGER,
    ExternalIdentifier TEXT,
    StoreID INTEGER,
    guid TEXT
);

CREATE TABLE ABPerson (
    ROWID INTEGER PRIMARY KEY AUTOINCREMENT,
    First TEXT,
    Last TEXT,
    Middle TEXT,
    FirstPhonetic TEXT,
    MiddlePhonetic TEXT,
    LastPhonetic TEXT,
    Organization TEXT,
    Department TEXT,
    Note TEXT,
    Kind INTEGER,
    Birthday TEXT,
    JobTitle TEXT,
    Nickname TEXT,
    Prefix TEXT,
    Suffix TEXT,
    FirstSort TEXT,
    LastSort TEXT,
    CreationDate INTEGER,
    ModificationDate INTEGER,
    CompositeNameFallback TEXT,
    ExternalIdentifier TEXT,
    ExternalModificationTag TEXT,
    ExternalUUID TEXT,
    StoreID INTEGER,
    DisplayName TEXT,
    ExternalRepresentation BLOB,
    FirstSortSection TEXT,
    LastSortSection TEXT,
    FirstSortLanguageIndex INTEGER DEFAULT 2147483647,
    LastSortLanguageIndex INTEGER DEFAULT 2147483647,
    PersonLink INTEGER DEFAULT -1,
    ImageURI TEXT,
    IsPreferredName INTEGER DEFAULT 1,
    guid TEXT NOT NULL,
    PhonemeData TEXT,
    AlternateBirthday TEXT,
    MapsData TEXT,
    FirstPronunciation TEXT,
    MiddlePronunciation TEXT,
    LastPronunciation TEXT,
    OrganizationPhonetic TEXT,
    OrganizationPronunciation TEXT,
    PreviousFamilyName TEXT,
    PreferredLikenessSource TEXT,
    PreferredPersonaIdentifier TEXT,
    PreferredChannel TEXT,
    DowntimeWhitelist TEXT,
    ImageType TEXT,
    ImageHash BLOB,
    MemojiMetadata BLOB,
    Wallpaper BLOB,
    DisplayFlags INTEGER,
    WatchWallpaperImageData BLOB,
    WallpaperMetadata BLOB,
    ImageBackgroundColorsData BLOB,
    SensitiveContentConfiguration BLOB,
    WallpaperURI TEXT,
    ImageSyncFailedTime TEXT,
    WallpaperSyncFailedTime TEXT,
    UNIQUE(guid)
);

CREATE TABLE ABPersonChanges (
    record INTEGER,
    type INTEGER,
    sequence_number INTEGER,
    Image INTEGER,
    ExternalIdentifier TEXT,
    StoreID INTEGER,
    PersonLink INTEGER,
    guid TEXT,
    Changes INTEGER,
    LinkToPersonUUID TEXT,
    PersonLinkUUID TEXT
);

CREATE TABLE ABPersonLink (
    ROWID INTEGER PRIMARY KEY AUTOINCREMENT,
    PreferredImagePersonID INTEGER DEFAULT -1,
    PreferredNamePersonID INTEGER DEFAULT -1,
    guid TEXT NOT NULL,
    UNIQUE(guid)
);

CREATE TABLE FirstSortSectionCount (
    LanguageIndex,
    Section TEXT,
    StoreID,
    number,
    UNIQUE(LanguageIndex, Section, StoreID)
);

CREATE TABLE LastSortSectionCount (
    LanguageIndex,
    Section TEXT,
    StoreID,
    number,
    UNIQUE(LanguageIndex, Section, StoreID)
);

CREATE TABLE FirstSortSectionCountTotal (
    LanguageIndex,
    Section TEXT,
    Type INTEGER,
    number,
    UNIQUE(LanguageIndex, Section, Type)
);

CREATE TABLE LastSortSectionCountTotal (
    LanguageIndex,
    Section TEXT,
    Type INTEGER,
    number,
    UNIQUE(LanguageIndex, Section, Type)
);

INSERT INTO ABStore (
    ROWID,
    Type,
    StoreInternalIdentifier,
    AccountID,
    Enabled,
    MeIdentifier,
    Capabilities,
    guid
)
VALUES (
    0,
    0,
    'local',
    -1,
    0,
    -1,
    0,
    '00000000-0000-4000-8000-000000000001'
);

INSERT INTO ABMultiValueLabel(value)
VALUES ('_$!<Home>!$_');

INSERT INTO ABMultiValueLabel(value)
VALUES ('_$!<Mobile>!$_');

INSERT INTO ABMultiValueLabel(value)
VALUES ('_$!<Other>!$_');

INSERT INTO ABMultiValueLabel(value)
VALUES ('_$!<Work>!$_');

INSERT INTO ABMultiValueLabel(value)
VALUES ('iPhone');

INSERT INTO ABMultiValueLabel(value)
VALUES ('');
    
COMMIT;