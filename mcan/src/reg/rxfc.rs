#[doc = "Register `RXFC` reader"]
pub type R = crate::R<RXFC_SPEC>;
#[doc = "Register `RXFC` writer"]
pub type W = crate::W<RXFC_SPEC>;
#[doc = "Field `FSA` reader - Rx FIFO Start Address"]
pub type FSA_R = crate::FieldReader<u16>;
#[doc = "Field `FSA` writer - Rx FIFO Start Address"]
pub type FSA_W<'a, REG, const O: u8> = crate::FieldWriter<'a, REG, 16, O, u16>;
#[doc = "Field `FS` reader - Rx FIFO Size"]
pub type FS_R = crate::FieldReader;
#[doc = "Field `FS` writer - Rx FIFO Size"]
pub type FS_W<'a, REG, const O: u8> = crate::FieldWriter<'a, REG, 7, O>;
#[doc = "Field `FWM` reader - Rx FIFO Watermark"]
pub type FWM_R = crate::FieldReader;
#[doc = "Field `FWM` writer - Rx FIFO Watermark"]
pub type FWM_W<'a, REG, const O: u8> = crate::FieldWriter<'a, REG, 7, O>;
#[doc = "Field `FOM` reader - FIFO Operation Mode"]
pub type FOM_R = crate::BitReader;
#[doc = "Field `FOM` writer - FIFO Operation Mode"]
pub type FOM_W<'a, REG, const O: u8> = crate::BitWriter<'a, REG, O>;
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
        FOM_R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Rx FIFO Start Address"]
    #[inline(always)]
    #[must_use]
    pub fn fsa(&mut self) -> FSA_W<RXFC_SPEC, 0> {
        FSA_W::new(self)
    }
    #[doc = "Bits 16:22 - Rx FIFO Size"]
    #[inline(always)]
    #[must_use]
    pub fn fs(&mut self) -> FS_W<RXFC_SPEC, 16> {
        FS_W::new(self)
    }
    #[doc = "Bits 24:30 - Rx FIFO Watermark"]
    #[inline(always)]
    #[must_use]
    pub fn fwm(&mut self) -> FWM_W<RXFC_SPEC, 24> {
        FWM_W::new(self)
    }
    #[doc = "Bit 31 - FIFO Operation Mode"]
    #[inline(always)]
    #[must_use]
    pub fn fom(&mut self) -> FOM_W<RXFC_SPEC, 31> {
        FOM_W::new(self)
    }
    #[doc = r" Writes raw bits to the register."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" Passing incorrect value can cause undefined behaviour. See reference manual"]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.bits = bits;
        self
    }
}
#[doc = "Rx FIFO Configuration\n\nYou can [`read`](crate::reg::generic::Reg::read) this register and get [`rxfc::R`](R).  You can [`reset`](crate::reg::generic::Reg::reset), [`write`](crate::reg::generic::Reg::write), [`write_with_zero`](crate::reg::generic::Reg::write_with_zero) this register using [`rxfc::W`](W). You can also [`modify`](crate::reg::generic::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RXFC_SPEC;
impl crate::RegisterSpec for RXFC_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rxfc::R`](R) reader structure"]
impl crate::Readable for RXFC_SPEC {}
#[doc = "`write(|w| ..)` method takes [`rxfc::W`](W) writer structure"]
impl crate::Writable for RXFC_SPEC {
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets RXFC to value 0"]
impl crate::Resettable for RXFC_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
