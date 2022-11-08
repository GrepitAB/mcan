#[doc = "Register `RXFC` reader"]
pub struct R(crate::R<RXFC_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<RXFC_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<RXFC_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<RXFC_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `RXFC` writer"]
pub struct W(crate::W<RXFC_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<RXFC_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<crate::W<RXFC_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<RXFC_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `FSA` reader - Rx FIFO Start Address"]
pub struct FSA_R(crate::FieldReader<u16, u16>);
impl FSA_R {
    #[inline(always)]
    pub(crate) fn new(bits: u16) -> Self {
        FSA_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FSA_R {
    type Target = crate::FieldReader<u16, u16>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FSA` writer - Rx FIFO Start Address"]
pub struct FSA_W<'a> {
    w: &'a mut W,
}
impl<'a> FSA_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | (value as u32 & 0xffff);
        self.w
    }
}
#[doc = "Field `FS` reader - Rx FIFO Size"]
pub struct FS_R(crate::FieldReader<u8, u8>);
impl FS_R {
    #[inline(always)]
    pub(crate) fn new(bits: u8) -> Self {
        FS_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FS_R {
    type Target = crate::FieldReader<u8, u8>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FS` writer - Rx FIFO Size"]
pub struct FS_W<'a> {
    w: &'a mut W,
}
impl<'a> FS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x7f << 16)) | ((value as u32 & 0x7f) << 16);
        self.w
    }
}
#[doc = "Field `FWM` reader - Rx FIFO Watermark"]
pub struct FWM_R(crate::FieldReader<u8, u8>);
impl FWM_R {
    #[inline(always)]
    pub(crate) fn new(bits: u8) -> Self {
        FWM_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FWM_R {
    type Target = crate::FieldReader<u8, u8>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FWM` writer - Rx FIFO Watermark"]
pub struct FWM_W<'a> {
    w: &'a mut W,
}
impl<'a> FWM_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x7f << 24)) | ((value as u32 & 0x7f) << 24);
        self.w
    }
}
#[doc = "Field `FOM` reader - FIFO Operation Mode"]
pub struct FOM_R(crate::FieldReader<bool, bool>);
impl FOM_R {
    #[inline(always)]
    pub(crate) fn new(bits: bool) -> Self {
        FOM_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FOM_R {
    type Target = crate::FieldReader<bool, bool>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FOM` writer - FIFO Operation Mode"]
pub struct FOM_W<'a> {
    w: &'a mut W,
}
impl<'a> FOM_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 31)) | ((value as u32 & 0x01) << 31);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:15 - Rx FIFO Start Address"]
    #[inline(always)]
    pub fn fsa(&self) -> FSA_R {
        FSA_R::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:22 - Rx FIFO Size"]
    #[inline(always)]
    pub fn fs(&self) -> FS_R {
        FS_R::new(((self.bits >> 16) & 0x7f) as u8)
    }
    #[doc = "Bits 24:30 - Rx FIFO Watermark"]
    #[inline(always)]
    pub fn fwm(&self) -> FWM_R {
        FWM_R::new(((self.bits >> 24) & 0x7f) as u8)
    }
    #[doc = "Bit 31 - FIFO Operation Mode"]
    #[inline(always)]
    pub fn fom(&self) -> FOM_R {
        FOM_R::new(((self.bits >> 31) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Rx FIFO Start Address"]
    #[inline(always)]
    pub fn fsa(&mut self) -> FSA_W {
        FSA_W { w: self }
    }
    #[doc = "Bits 16:22 - Rx FIFO Size"]
    #[inline(always)]
    pub fn fs(&mut self) -> FS_W {
        FS_W { w: self }
    }
    #[doc = "Bits 24:30 - Rx FIFO Watermark"]
    #[inline(always)]
    pub fn fwm(&mut self) -> FWM_W {
        FWM_W { w: self }
    }
    #[doc = "Bit 31 - FIFO Operation Mode"]
    #[inline(always)]
    pub fn fom(&mut self) -> FOM_W {
        FOM_W { w: self }
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Rx FIFO Configuration\n\nThis register you can [`read`](crate::reg::generic::Reg::read), [`write_with_zero`](crate::reg::generic::Reg::write_with_zero), [`reset`](crate::reg::generic::Reg::reset), [`write`](crate::reg::generic::Reg::write), [`modify`](crate::reg::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rxfc](index.html) module"]
pub struct RXFC_SPEC;
impl crate::RegisterSpec for RXFC_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [rxfc::R](R) reader structure"]
impl crate::Readable for RXFC_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [rxfc::W](W) writer structure"]
impl crate::Writable for RXFC_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets RXFC to value 0"]
impl crate::Resettable for RXFC_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
