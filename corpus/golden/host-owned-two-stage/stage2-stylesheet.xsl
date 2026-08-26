<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet
  version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="xml" omit-xml-declaration="yes"/>

  <xsl:template match="/">
    <stage-two><xsl:value-of select="message"/></stage-two>
  </xsl:template>
</xsl:stylesheet>
