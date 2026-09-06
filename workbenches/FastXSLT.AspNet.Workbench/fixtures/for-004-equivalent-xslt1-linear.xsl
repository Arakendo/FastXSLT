<?xml version="1.0"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:output method="xml" encoding="UTF-8" omit-xml-declaration="no"/>
  <xsl:template match="order">
    <out>
      <xsl:call-template name="total">
        <xsl:with-param name="item" select="order-item[1]"/>
      </xsl:call-template>
    </out>
  </xsl:template>
  <xsl:template name="total">
    <xsl:param name="item"/>
    <xsl:param name="sum" select="0"/>
    <xsl:choose>
      <xsl:when test="$item">
        <xsl:call-template name="total">
          <xsl:with-param name="item" select="$item/following-sibling::order-item[1]"/>
          <xsl:with-param name="sum" select="$sum + $item/@price * $item/@qty"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise>
        <xsl:value-of select="format-number($sum, '0.00')"/>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>
</xsl:stylesheet>
