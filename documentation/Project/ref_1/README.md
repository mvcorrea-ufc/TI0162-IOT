# ESP32-C3 IoT System Technical Documentation

This directory contains the comprehensive LaTeX-based technical documentation for the ESP32-C3 Modular IoT System project.

## Document Overview

The technical documentation provides:

- **System Architecture**: Complete modular design overview with TikZ diagrams
- **Hardware Configuration**: Pin assignments, connections, and specifications  
- **Software Implementation**: Rust/Embassy framework integration patterns
- **Module Documentation**: Detailed description of each system component
- **Future Enhancements**: PID control system and expansion plans
- **Performance Specifications**: System metrics and sensor accuracy
- **Build Instructions**: Complete setup and deployment guide

## Using with Overleaf

This documentation is designed for use with Overleaf:

1. **Copy `main.tex` content** to a new Overleaf project
2. **Compile directly** - all required packages are standard
3. **TikZ diagrams** will render automatically
4. **No additional setup** required

### Required LaTeX Packages

The document uses standard packages available in Overleaf:
- `tikz` - For system architecture diagrams
- `pgfplots` - For performance charts  
- `listings` - For code syntax highlighting
- `hyperref` - For PDF navigation and links
- `booktabs` - For professional tables

## Document Structure

```
ref_1/
├── main.tex           # Main LaTeX document
├── Makefile          # Build automation
├── README.md         # This file
└── build/            # Generated build artifacts (created during build)
```

## Key Features

### TikZ Diagrams

The documentation includes professional system diagrams created with TikZ:

1. **System Architecture Overview**: Shows the complete modular structure
2. **Hardware Connection Diagrams**: BME280 sensor wiring
3. **Future PID Control Loop**: Planned environmental control system

### Code Integration

- Rust code examples with syntax highlighting
- JSON message format specifications
- Configuration file templates
- Build script examples

### Professional Formatting

- Academic paper style with proper citations
- Table of contents with hyperlinks
- Professional tables with booktabs
- Consistent formatting throughout

## Output

The build process generates:
- `ESP32-C3_IoT_Technical_Documentation.pdf` - The final documentation

## Document Sections

1. **Introduction** - Project overview and key features
2. **System Architecture** - Modular design and component interaction
3. **Hardware Configuration** - Pin assignments and connections
4. **Software Implementation** - Rust/Embassy integration patterns
5. **Data Flow and Communication** - MQTT protocols and message formats
6. **Future Enhancements** - PID control and expansion plans
7. **Performance and Specifications** - System metrics and capabilities
8. **Conclusion** - Project summary and design principles
9. **References** - Technical references and standards
10. **Appendices** - Build instructions and configuration files

## Maintenance

To update the documentation:

1. Edit `main.tex` with your changes
2. Run `make rebuild` to ensure clean build
3. Review the generated PDF for formatting
4. Commit changes to version control

## Integration with Project

This documentation is designed to complement the project's markdown documentation while providing:

- **Academic rigor** for technical presentations
- **Professional formatting** for reports and proposals  
- **Detailed diagrams** using TikZ for precision
- **Print-ready output** for physical documentation needs

The LaTeX source follows the project's documentation standards:
- English language only
- Technical accuracy preservation
- Professional presentation standards
- Comprehensive coverage of all modules