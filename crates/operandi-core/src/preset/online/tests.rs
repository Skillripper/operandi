use super::*;
use crate::preset::library::{self, ListOptions};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "operandi-online-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Two HDRIs as Poly Haven's `/assets?type=hdris` lists them (cut down).
const PH_HDRIS: &str = r#"{
  "aarfontein_dawn": {"type": 0, "name": "Aarfontein Dawn",
    "category": "Desert & Arid/Karoo & Scrubland/Rocky Outcrops",
    "tags": ["dawn", "rocky"], "authors": {"Jarod Guest": "processing", "Greg Zaal": "photography"},
    "max_resolution": [24576, 12288],
    "thumbnail_url": "https://cdn.polyhaven.com/asset_img/thumbs/aarfontein_dawn.png?width=256&height=256&format=webp&v=7f3a"},
  "old_one": {"type": 0, "name": "Old One", "categories": ["outdoor", "skies"],
    "authors": {"Someone": "All"}, "max_resolution": [4096, 2048]},
  "../evil": {"type": 0, "name": "Evil"}
}"#;

/// One HDRI's `/files/<id>` answer (cut down).
const PH_HDRI_FILES: &str = r#"{
  "hdri": {
    "1k": {"hdr": {"url": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/x_1k.hdr", "size": 1500000, "md5": "a"},
           "exr": {"url": "https://dl.polyhaven.org/file/ph-assets/HDRIs/exr/1k/x_1k.exr", "size": 2500000, "md5": "b"}},
    "4k": {"hdr": {"url": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/4k/x_4k.hdr", "size": 24000000, "md5": "c"}}
  },
  "tonemapped": {"url": "https://dl.polyhaven.org/x.jpg", "size": 1, "md5": "d"}
}"#;

const PH_TEXTURE_FILES: &str = r#"{
  "Diffuse": {"1k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/1k/rock/rock_diff_1k.jpg", "size": 800, "md5": ""},
                     "png": {"url": "https://dl.polyhaven.org/Textures/png/1k/rock/rock_diff_1k.png", "size": 6000, "md5": ""}},
              "2k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/2k/rock/rock_diff_2k.jpg", "size": 3000, "md5": ""}}},
  "nor_gl": {"2k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/2k/rock/rock_nor_gl_2k.jpg", "size": 2000, "md5": ""}}},
  "nor_dx": {"2k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/2k/rock/rock_nor_dx_2k.jpg", "size": 2000, "md5": ""}}},
  "Rough": {"2k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/2k/rock/rock_rough_2k.jpg", "size": 1000, "md5": ""}}},
  "Displacement": {"2k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/2k/rock/rock_disp_2k.jpg", "size": 500, "md5": ""},
                          "exr": {"url": "https://dl.polyhaven.org/Textures/exr/2k/rock/rock_disp_2k.exr", "size": 900, "md5": ""}}},
  "arm": {"2k": {"jpg": {"url": "https://dl.polyhaven.org/Textures/jpg/2k/rock/rock_arm_2k.jpg", "size": 1000, "md5": ""}}},
  "gltf": {"2k": {"gltf": {"url": "https://dl.polyhaven.org/x.gltf", "size": 3, "md5": "", "include": {}}}}
}"#;

const PH_MODEL_FILES: &str = r#"{
  "gltf": {"1k": {"gltf": {"url": "https://dl.polyhaven.org/Models/gltf/1k/chair/chair_1k.gltf", "size": 30, "md5": "",
     "include": {
       "chair.bin": {"url": "https://dl.polyhaven.org/Models/gltf/chair/chair.bin", "size": 2000, "md5": ""},
       "textures/chair_diff_1k.jpg": {"url": "https://dl.polyhaven.org/Models/jpg/1k/chair/chair_diff_1k.jpg", "size": 500, "md5": ""}
     }}}}
}"#;

/// One ambientCG `/api/v3/assets` page (cut down).
const ACG_MATERIALS: &str = r#"{
  "totalResults": 2,
  "nextPageHttp": "https://ambientcg.com/api/v3/assets?type=material&limit=250&offset=250",
  "assets": [
    {"id": "PavingStones142", "title": "Paving Stones 142", "type": "material",
     "downloads": [
       {"attributes": "1K-JPG", "extension": "zip", "url": "https://ambientcg.com/get?file=PavingStones142_1K-JPG.zip", "size": 3000000},
       {"attributes": "1K-PNG", "extension": "zip", "url": "https://ambientcg.com/get?file=PavingStones142_1K-PNG.zip", "size": 7000000},
       {"attributes": "4K-JPG", "extension": "zip", "url": "https://ambientcg.com/get?file=PavingStones142_4K-JPG.zip", "size": 36000000}
     ],
     "thumbnails": {"256-PNG": "https://acg-media.struffelproductions.com/file/ambientCG-Web/media/thumbnail/256-PNG/PavingStones142.png",
                    "256-JPG-242424": "https://acg-media.struffelproductions.com/file/ambientCG-Web/media/thumbnail/256-JPG-242424/PavingStones142.jpg"}},
    {"id": "NoFiles001", "title": "No Files", "downloads": []}
  ]
}"#;

#[test]
fn poly_haven_catalogue_reads_names_categories_authors_and_thumbnails() {
    let assets = parse_polyhaven(AssetKind::Hdri, PH_HDRIS).unwrap();
    // The unsafe id is left out.
    assert_eq!(assets.len(), 2);
    let dawn = &assets[0];
    assert_eq!(dawn.id, "aarfontein_dawn");
    assert_eq!(dawn.name, "Aarfontein Dawn");
    assert_eq!(dawn.category, "Desert & Arid");
    assert_eq!(dawn.authors, ["Greg Zaal", "Jarod Guest"]);
    assert_eq!(dawn.max_k, Some(24));
    assert_eq!(dawn.source, SourceId::PolyHaven);
    // A JPEG thumbnail: Operandi doesn't read WebP.
    assert!(dawn.thumbnail.contains("format=jpg"), "{}", dawn.thumbnail);
    assert!(!dawn.thumbnail.contains("webp"));
    assert!(dawn.thumbnail.contains("v=7f3a"));
    // No category path: its first old-style tag, capitalised.
    assert_eq!(assets[1].category, "Outdoor");
    assert_eq!(assets[1].resolutions(), [1, 2, 4]);
    assert!(parse_polyhaven(AssetKind::Hdri, "[]").is_err());
}

#[test]
fn ambientcg_catalogue_keeps_jpg_material_archives_and_the_next_page() {
    let (assets, next) = parse_ambientcg(AssetKind::Texture, ACG_MATERIALS).unwrap();
    assert_eq!(assets.len(), 1, "an asset without archives is left out");
    let a = &assets[0];
    assert_eq!(a.category, "Paving Stones");
    assert_eq!(a.files.iter().map(|f| f.k).collect::<Vec<_>>(), [1, 4]);
    assert!(a.files.iter().all(|f| f.label.ends_with("-JPG")));
    assert_eq!(a.resolutions(), [1, 4]);
    assert!(a.thumbnail.ends_with("256-JPG-242424/PavingStones142.jpg"));
    assert_eq!(
        next.as_deref(),
        Some("https://ambientcg.com/api/v3/assets?type=material&limit=250&offset=250")
    );
    // A next page somewhere else is never followed.
    let elsewhere = ACG_MATERIALS.replace("https://ambientcg.com/api", "https://example.com/api");
    assert_eq!(
        parse_ambientcg(AssetKind::Texture, &elsewhere).unwrap().1,
        None
    );
}

#[test]
fn resolution_is_the_highest_at_or_below_the_wanted_else_the_lowest() {
    assert_eq!(pick_k(&[1, 2, 4, 8], 4), Some(4));
    assert_eq!(pick_k(&[1, 4, 8], 2), Some(1));
    assert_eq!(pick_k(&[4, 8], 2), Some(4));
    assert_eq!(pick_k(&[1, 2], 16), Some(2));
    assert_eq!(pick_k(&[], 2), None);
}

#[test]
fn poly_haven_hdri_plan_is_one_hdr_file() {
    let asset = &parse_polyhaven(AssetKind::Hdri, PH_HDRIS).unwrap()[0];
    let plan = polyhaven_plan(asset, PH_HDRI_FILES, 2).unwrap();
    assert_eq!(plan.k, 1);
    assert_eq!(plan.files.len(), 1);
    assert!(plan.files[0].url.ends_with("x_1k.hdr"));
    assert_eq!(
        plan.main.as_deref(),
        Some(Path::new("aarfontein_dawn_1k.hdr"))
    );
    assert_eq!(plan.bytes(), 1_500_000);
    let plan = polyhaven_plan(asset, PH_HDRI_FILES, 8).unwrap();
    assert_eq!(plan.k, 4);
}

#[test]
fn poly_haven_texture_plan_fetches_one_file_per_map() {
    let mut asset = parse_polyhaven(AssetKind::Hdri, PH_HDRIS)
        .unwrap()
        .remove(0);
    asset.kind = AssetKind::Texture;
    let plan = polyhaven_plan(&asset, PH_TEXTURE_FILES, 2).unwrap();
    let names: Vec<String> = plan
        .files
        .iter()
        .map(|f| f.rel.display().to_string())
        .collect();
    assert_eq!(
        names,
        [
            "rock_diff_2k.jpg",
            "rock_nor_gl_2k.jpg",
            "rock_rough_2k.jpg",
            "rock_disp_2k.jpg"
        ],
        "no packed, DirectX or glTF files"
    );
    assert_eq!(plan.main.as_deref(), Some(Path::new("rock_diff_2k.jpg")));
    // The names are ones the texture-set reader knows.
    let paths: Vec<PathBuf> = plan.files.iter().map(|f| f.rel.clone()).collect();
    let refs: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    let sets = library::texture_sets(&refs);
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].maps.len(), 4);
}

#[test]
fn poly_haven_model_plan_keeps_the_gltf_beside_its_files() {
    let mut asset = parse_polyhaven(AssetKind::Hdri, PH_HDRIS)
        .unwrap()
        .remove(0);
    asset.kind = AssetKind::Model;
    let plan = polyhaven_plan(&asset, PH_MODEL_FILES, 2).unwrap();
    assert_eq!(plan.k, 1);
    let rels: Vec<PathBuf> = plan.files.iter().map(|f| f.rel.clone()).collect();
    assert_eq!(rels[0], Path::new("chair_1k.gltf"));
    assert!(rels.contains(&PathBuf::from("chair.bin")));
    assert!(rels.contains(&Path::new("textures").join("chair_diff_1k.jpg")));
    // A listed file reaching outside its folder refuses the download.
    let evil = PH_MODEL_FILES.replace("textures/chair_diff_1k.jpg", "../../etc/x.jpg");
    assert!(polyhaven_plan(&asset, &evil, 1).is_err());
}

#[test]
fn unsafe_relative_paths_are_refused() {
    assert_eq!(
        safe_rel("textures/a.jpg"),
        Some(Path::new("textures").join("a.jpg"))
    );
    assert_eq!(safe_rel("./a.bin"), Some(PathBuf::from("a.bin")));
    assert_eq!(safe_rel("../a"), None);
    assert_eq!(safe_rel("/etc/passwd"), None);
    assert_eq!(safe_rel("a/../../b"), None);
    assert_eq!(safe_rel(""), None);
}

#[test]
fn stubs_list_as_entries_of_their_kind_and_sync_removes_only_stubs() {
    let root = temp_dir("stubs");
    let assets = parse_polyhaven(AssetKind::Hdri, PH_HDRIS).unwrap();
    let report = sync_stubs(&root, &assets).unwrap();
    assert_eq!(
        report,
        SyncReport {
            written: 2,
            removed: 0
        }
    );
    // A second sync with nothing new writes nothing.
    assert_eq!(sync_stubs(&root, &assets).unwrap(), SyncReport::default());
    let folder = root.join("HDRIs").join("Desert & Arid");
    let listing =
        library::list_folder(&folder, ListOptions::default(), &|_| None, &|| false).unwrap();
    assert_eq!(listing.entries.len(), 1);
    let e = &listing.entries[0];
    assert_eq!(e.name, "Aarfontein Dawn");
    assert_eq!(e.kind, EntryKind::Environment);
    assert_eq!(
        e.online.as_deref().map(|a| a.id.as_str()),
        Some("aarfontein_dawn")
    );
    assert!(e.in_bucket(library::Bucket::Environments));
    // Someone's own file in the cache is never removed.
    std::fs::write(folder.join("mine.txt"), b"keep").unwrap();
    let report = sync_stubs(&root, &assets[1..]).unwrap();
    assert_eq!(report.removed, 1);
    assert!(folder.join("mine.txt").is_file());
    assert!(!stub_path(&root, &assets[0]).exists());
    // A category left empty goes.
    std::fs::remove_file(folder.join("mine.txt")).unwrap();
    sync_stubs(&root, &assets[1..]).unwrap();
    assert!(!folder.exists());
    assert!(root.join("HDRIs").join("Outdoor").is_dir());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn same_names_in_one_folder_both_get_stubs() {
    let root = temp_dir("names");
    let mut assets = parse_polyhaven(AssetKind::Hdri, PH_HDRIS).unwrap();
    assets[1].name = assets[0].name.clone();
    assets[1].category = assets[0].category.clone();
    sync_stubs(&root, &assets).unwrap();
    assert_eq!(stubs_below(&root.join("HDRIs")).len(), 2);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn downloads_count_only_once_marked_complete() {
    let root = temp_dir("downloads");
    let asset = parse_polyhaven(AssetKind::Hdri, PH_HDRIS)
        .unwrap()
        .remove(0);
    let dir = download_dir(&root, &asset, 2);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("aarfontein_dawn_2k.hdr");
    std::fs::write(&file, b"#?RADIANCE").unwrap();
    assert!(downloaded(&root, &asset).is_empty(), "no marker yet");
    finish_download(&dir, &file).unwrap();
    assert_eq!(downloaded(&root, &asset), [(2, file.clone())]);
    assert_eq!(downloaded_at(&root, &asset, 8), Some((2, file.clone())));
    assert_eq!(downloaded_at(&root, &asset, 1), Some((2, file)));
    // The cache's hidden folders never list.
    let listing =
        library::list_folder(&root, ListOptions::default(), &|_| None, &|| false).unwrap();
    assert!(listing.folders.is_empty() && listing.entries.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ambientcg_archives_unpack_their_maps_and_name_the_file_to_load() {
    let (assets, _) = parse_ambientcg(AssetKind::Texture, ACG_MATERIALS).unwrap();
    let asset = &assets[0];
    let plan = ambientcg_plan(asset, 2).unwrap();
    assert!(plan.archive && plan.main.is_none());
    assert_eq!(plan.k, 1);
    let zip = crate::interchange::zip::write(&[
        ("PavingStones142_1K-JPG_Color.jpg", b"c".as_slice()),
        ("PavingStones142_1K-JPG_NormalGL.jpg", b"n".as_slice()),
        ("PavingStones142_1K-JPG.blend", b"b".as_slice()),
        ("PavingStones142_1K-JPG.usdc", b"u".as_slice()),
        ("PavingStones142.png", b"preview".as_slice()),
        ("../escape.jpg", b"x".as_slice()),
    ])
    .unwrap();
    let dir = temp_dir("unpack");
    let main = unpack(&zip, &dir, asset).unwrap();
    assert_eq!(main, dir.join("PavingStones142_1K-JPG_Color.jpg"));
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "PavingStones142_1K-JPG_Color.jpg",
            "PavingStones142_1K-JPG_NormalGL.jpg"
        ]
    );
    assert!(!dir.parent().unwrap().join("escape.jpg").exists());
    // An HDRI archive without a panorama loads nothing.
    let mut hdri = asset.clone();
    hdri.kind = AssetKind::Hdri;
    let only_jpg = crate::interchange::zip::write(&[
        ("Sky_1K_TONEMAPPED.jpg", b"t".as_slice()),
        ("x.jpg", b"x".as_slice()),
    ])
    .unwrap();
    assert!(unpack(&only_jpg, &temp_dir("unpack-hdri"), &hdri).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn catalogues_round_trip_and_age() {
    let root = temp_dir("catalogue");
    let assets = parse_polyhaven(AssetKind::Hdri, PH_HDRIS).unwrap();
    let c = Catalogue::new(SourceId::PolyHaven, 1_000, assets);
    c.write(&root).unwrap();
    let back = Catalogue::read(&root).unwrap();
    assert_eq!(back, c);
    assert!(back.find(AssetKind::Hdri, "old_one").is_some());
    let week = 7 * 86_400;
    assert!(!back.stale(1_000 + week - 1, Some(week)));
    assert!(back.stale(1_000 + week, Some(week)));
    assert!(!back.stale(u64::MAX, None), "only by hand");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn names_and_slugs() {
    assert_eq!(SourceId::from_slug("Poly Haven"), Some(SourceId::PolyHaven));
    assert_eq!(SourceId::from_slug("ambientCG"), Some(SourceId::AmbientCg));
    assert_eq!(SourceId::from_slug("nope"), None);
    assert_eq!(AssetKind::from_slug("HDRIs"), Some(AssetKind::Hdri));
    assert_eq!(AssetKind::from_slug("materials"), Some(AssetKind::Texture));
    assert_eq!(AssetKind::from_slug("model"), Some(AssetKind::Model));
    assert_eq!(file_name_for("a/b: c?"), "a-b- c-");
    assert_eq!(file_name_for(" .. "), "Untitled");
    assert!(SourceId::PolyHaven.credit().contains("Poly Haven"));
}
