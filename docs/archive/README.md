# Archived Documentation

This directory contains the original documentation files that have been reorganized into the new `docs/` structure.

## Archived Files

### DEBUG_SETUP.md
**Migrated to**: [`docs/developer-guide/debugging.md`](../developer-guide/debugging.md)

Original VSCode debugging setup guide. Content has been expanded and integrated into the new debugging guide.

### MAP_LOADER_DESIGN.md
**Migrated to**: [`docs/developer-guide/systems/map-loader.md`](../developer-guide/systems/map-loader.md)

Original map loader system architecture document. Technical content has been preserved and enhanced in the new system documentation.

### MAP_LOADER_USAGE.md
**Migrated to**: Multiple locations:
- [`docs/user-guide/maps/creating-maps.md`](../user-guide/maps/creating-maps.md) - User-focused creation guide
- [`docs/user-guide/maps/map-format.md`](../user-guide/maps/map-format.md) - Format reference
- [`docs/user-guide/maps/examples.md`](../user-guide/maps/examples.md) - Example walkthroughs

Original usage guide split into focused, audience-specific documents.

### README_MAP_LOADER.md
**Migrated to**: Multiple locations:
- [`docs/developer-guide/systems/map-loader.md`](../developer-guide/systems/map-loader.md) - Implementation details
- [`docs/user-guide/maps/creating-maps.md`](../user-guide/maps/creating-maps.md) - Usage examples

Implementation summary distributed across appropriate documentation sections.

## Why Archive?

These files are preserved for:
1. **Historical Reference** - Track documentation evolution
2. **Content Verification** - Ensure no information was lost during migration
3. **Rollback Option** - Revert if needed (though not recommended)

## New Documentation Structure

The documentation has been reorganized into a clear hierarchy:

```
docs/
├── README.md                    # Documentation hub
├── getting-started/             # New user guides
├── user-guide/                  # Player and map creator docs
├── developer-guide/             # Developer documentation
├── api/                         # Technical specifications
└── archive/                     # This directory
```

See [`docs/README.md`](../README.md) for the complete documentation index.

## Migration Date

**Date**: 2025-01-10  
**Reason**: Improve organization, discoverability, and maintainability

---

**Note**: These archived files are kept for reference only. Please use the new documentation structure for all current information.