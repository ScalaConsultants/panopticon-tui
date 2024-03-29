# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [0.2.1]
### Changed
Bumped jmx to 0.2.1 which should fix the issue with auto-locating jdk on MacOS

## [0.2.0]
### Added
- Akka deadletters metrics (via [akka-periscope](https://github.com/ScalaConsultants/akka-periscope))
- Akka actor system start time and uptime (via [akka-periscope](https://github.com/ScalaConsultants/akka-periscope))
### Changed
- `--actor-count` flag renamed into `--actor-system-status`.

## [0.1.1] - 2020-05-25
### Fixed
- Now panopticon will work with a single Akka tab

## [0.1.0] - 2020-05-19
### Added
- ZIO-ZMX integration
- Slick and HikariCP metrics
- Akka metrics integration (via [akka-periscope](https://github.com/ScalaConsultants/akka-periscope))
