// Included at the top of `ModuleFactory::newModule` in the census engine only
// (`scripts/module_census.py`). One line per module instance, on stderr.
if (!name.isEmpty())
	fprintf(stderr, "ZH_CENSUS %d %s\n", static_cast<int>(type), name.str());
