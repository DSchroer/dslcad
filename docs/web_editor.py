import logging, re
import shutil

log = logging.getLogger('mkdocs')
def on_post_build(config, **kwargs):
    site_dir = config["site_dir"] + "/editor/"
    editor_dir = config["extra"]["editor_dir"]
    shutil.copytree(editor_dir, site_dir, ignore=shutil.ignore_patterns("index.html"), dirs_exist_ok=True)
